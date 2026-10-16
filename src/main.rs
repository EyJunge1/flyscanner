mod aircraft_db;
mod api;
mod beast;
mod config_parse;
mod decode;
mod demo;
mod feeder;
mod history;
mod icao;
mod metrics;
mod state;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::api::{build_router, new_state, WsMessage};
use crate::beast::{
    normalize_uuid, spawn_forwarder, BeastClient, BeastOutServer, BeastSource, ForwardOpts,
};
use crate::config_parse::parse_flyscanner_config;
use crate::decode::{AdsbDecoder, MessageSource};
use crate::feeder::{spawn_sources, Deduper, MlatHub, SourceConfig};
use crate::history::HistoryStore;
use crate::metrics::{range_nm, Metrics};
use crate::icao::Icao24;
use crate::state::{now_secs, AircraftStore};
use anyhow::Result;
use clap::Parser;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(
    name = "flyscanner",
    about = "Modern ADS-B all-in-one (Rust + Svelte 5)"
)]
struct Args {
    /// HTTP listen address
    #[arg(long, env = "FLYSCANNER_LISTEN", default_value = "0.0.0.0:8080")]
    listen: SocketAddr,

    /// Beast TCP host (single source). Ignored if --demo or --beast-source is set.
    #[arg(long, env = "FLYSCANNER_BEAST_HOST", default_value = "127.0.0.1")]
    beast_host: String,

    /// Beast TCP port
    #[arg(long, env = "FLYSCANNER_BEAST_PORT", default_value_t = 30005)]
    beast_port: u16,

    /// Extra Beast sources: name=host:port[:mlat] (repeatable)
    #[arg(
        long = "beast-source",
        env = "FLYSCANNER_BEAST_SOURCES",
        value_delimiter = ','
    )]
    beast_sources: Vec<String>,

    /// Beast-out listen address (clients / feeder sidecars connect here)
    #[arg(long, env = "FLYSCANNER_BEAST_OUT", default_value = "0.0.0.0:30004")]
    beast_out: SocketAddr,

    /// Disable beast-out listener
    #[arg(long, env = "FLYSCANNER_NO_BEAST_OUT", default_value_t = false)]
    no_beast_out: bool,

    /// Ultrafeeder-like config: `adsb,host,port,beast_out;adsb,host,port,beast_in;...`
    #[arg(long = "config", env = "FLYSCANNER_CONFIG", default_value = "")]
    config: String,

    /// Receiver latitude
    #[arg(long, env = "FLYSCANNER_LAT", default_value_t = 52.52)]
    lat: f64,

    /// Receiver longitude
    #[arg(long, env = "FLYSCANNER_LON", default_value_t = 13.405)]
    lon: f64,

    /// Aircraft TTL seconds
    #[arg(long, default_value_t = 60)]
    ttl: u64,

    /// SQLite history path
    #[arg(long, env = "FLYSCANNER_DB", default_value = "data/flyscanner.db")]
    db: PathBuf,

    /// Aircraft metadata CSV (tar1090-db / readsb): path to `aircraft.csv` or `aircraft.csv.gz`
    #[arg(long, env = "FLYSCANNER_AIRCRAFT_DB")]
    aircraft_db: Option<PathBuf>,

    /// Run synthetic demo traffic (no Beast required)
    #[arg(long, env = "FLYSCANNER_DEMO", default_value_t = false)]
    demo: bool,

    /// Dedup window in milliseconds
    #[arg(long, default_value_t = 800)]
    dedup_ms: u64,

    /// History snapshot interval seconds
    #[arg(long, default_value_t = 8)]
    snapshot_interval: u64,

    /// Station UUID for `beast_reduce_plus_out` (env `FLYSCANNER_UUID` or `UUID`)
    #[arg(long, env = "FLYSCANNER_UUID")]
    uuid: Option<String>,

    /// Beast-reduce base interval seconds (Ultrafeeder default 0.5)
    #[arg(long, env = "FLYSCANNER_BEAST_REDUCE_INTERVAL", default_value_t = 0.5)]
    beast_reduce_interval: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "flyscanner=info,tower_http=info".into()),
        )
        .init();

    let args = Args::parse();
    info!(?args.listen, demo = args.demo, "starting flyscanner");

    let store = Arc::new(RwLock::new(AircraftStore::new(args.ttl, 2000)));
    let history = Arc::new(HistoryStore::open(&args.db).await?);

    // Aircraft DB: CSV (tar1090-db) as base, Turso overrides on top.
    let mut aircraft_meta = if let Some(path) = &args.aircraft_db {
        aircraft_db::load_from_path(path)?
    } else {
        std::collections::HashMap::new()
    };
    if aircraft_meta.is_empty() && args.demo {
        seed_sample_db(&history).await?;
    }
    match history.load_aircraft_db().await {
        Ok(overrides) => {
            for (icao, entry) in overrides {
                aircraft_meta.insert(icao, entry);
            }
        }
        Err(e) => warn!(error = %e, "failed loading turso aircraft_db overrides"),
    }
    let db_len = aircraft_meta.len();
    store.write().await.load_db(aircraft_meta);
    info!(entries = db_len, "aircraft metadata ready");

    let metrics = Arc::new(Metrics::new()?);
    let state = new_state(
        store.clone(),
        Some(history.clone()),
        metrics.clone(),
        args.lat,
        args.lon,
    );

    let parsed_cfg = parse_flyscanner_config(&args.config);
    for skip in &parsed_cfg.skipped {
        warn!(entry = %skip, "FLYSCANNER_CONFIG entry skipped (use feeder sidecars for mlat/mlathub)");
    }

    let shutdown = CancellationToken::new();
    let mut tasks = JoinSet::new();

    // Beast out hub (listen + optional outbound forwards from FLYSCANNER_CONFIG)
    let beast_hub = Arc::new(BeastOutServer::new(1024));
    if !args.no_beast_out {
        let s = beast_hub.clone();
        let addr = args.beast_out;
        let cancel = shutdown.clone();
        tasks.spawn(async move {
            if let Err(e) = s.listen(addr, cancel).await {
                warn!(error = %e, "beast-out failed");
            }
        });
    }
    let uuid_fallback = std::env::var("UUID").ok();
    let station_uuid = normalize_uuid(
        args.uuid
            .as_deref()
            .or(uuid_fallback.as_deref())
            .filter(|s| !s.is_empty()),
    );
    info!(uuid = %station_uuid, "station uuid ready (beast_reduce_plus)");

    for target in &parsed_cfg.feed_out {
        let opts = ForwardOpts {
            mode: target.mode,
            uuid: Some(target.uuid.clone().unwrap_or_else(|| station_uuid.clone())),
            reduce_interval: args.beast_reduce_interval,
        };
        spawn_forwarder(
            target.host.clone(),
            target.port,
            beast_hub.publisher(),
            opts,
            shutdown.clone(),
        );
        info!(
            host = %target.host,
            port = target.port,
            mode = ?target.mode,
            "beast-forward target"
        );
    }
    let beast_out = Some(beast_hub);

    // Ingest pipeline
    let (frame_tx, mut frame_rx) = mpsc::channel::<IngestFrame>(4096);

    if args.demo {
        demo::spawn(frame_tx.clone(), args.lat, args.lon, shutdown.clone());
    } else {
        let mut sources = parse_sources(&args.beast_sources);
        sources.extend(parsed_cfg.ingest.clone());
        if sources.is_empty() {
            sources.push(SourceConfig {
                name: "primary".into(),
                host: args.beast_host.clone(),
                port: args.beast_port,
                is_mlat: false,
            });
        }

        match sources.as_slice() {
            [single] if !single.is_mlat => {
                let (tx, mut rx) = mpsc::channel(1024);
                BeastClient::spawn(
                    BeastSource::new(single.host.clone(), single.port),
                    tx,
                    Some(metrics.clone()),
                    shutdown.clone(),
                );
                let out = frame_tx.clone();
                let cancel = shutdown.clone();
                tasks.spawn(async move {
                    loop {
                        tokio::select! {
                            () = cancel.cancelled() => break,
                            frame = rx.recv() => {
                                let Some(frame) = frame else { break };
                                if out
                                    .send(IngestFrame {
                                        frame,
                                        message_source: MessageSource::Adsb,
                                        is_demo: false,
                                        source: std::sync::Arc::from("primary"),
                                    })
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                        }
                    }
                });
            }
            _ => {
                let mut tagged_rx =
                    spawn_sources(sources, Some(metrics.clone()), shutdown.clone());
                let out = frame_tx.clone();
                let cancel = shutdown.clone();
                tasks.spawn(async move {
                    loop {
                        tokio::select! {
                            () = cancel.cancelled() => break,
                            t = tagged_rx.recv() => {
                                let Some(t) = t else { break };
                                if out
                                    .send(IngestFrame {
                                        frame: t.frame,
                                        message_source: t.message_source,
                                        is_demo: false,
                                        source: t.source,
                                    })
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                        }
                    }
                });
            }
        }
    }

    // Decode + state loop
    {
        let store = store.clone();
        let metrics = metrics.clone();
        let beast_out = beast_out.clone();
        let mut decoder = AdsbDecoder::new(Some((args.lat, args.lon)));
        let mut deduper = Deduper::new(args.dedup_ms);
        let mut mlat_hub = MlatHub::new();
        let receiver_lat = args.lat;
        let receiver_lon = args.lon;
        let cancel = shutdown.clone();

        tasks.spawn(async move {
            loop {
                tokio::select! {
                    () = cancel.cancelled() => break,
                    ingest = frame_rx.recv() => {
                        let Some(ingest) = ingest else { break };
                        if let Some(server) = &beast_out {
                            server.publish_frame(&ingest.frame);
                        }

                        let Some(payload) = ingest.frame.mode_s_payload() else {
                            continue;
                        };
                        let now = now_secs();
                        if !deduper.accept(payload, now) {
                            continue;
                        }

                        let t0 = std::time::Instant::now();

                        let decoded = if ingest.is_demo {
                            Ok(demo::try_parse_demo(&ingest.frame))
                        } else {
                            decoder.decode_frame(&ingest.frame, now)
                        };

                        match decoded {
                            Ok(Some(mut update)) => {
                                if ingest.message_source == MessageSource::Mlat {
                                    update.source = MessageSource::Mlat;
                                    if !mlat_hub.prefer_mlat(update.icao24, now, 15.0)
                                        && update.latitude.is_some()
                                    {
                                        continue;
                                    }
                                } else if update.latitude.is_some() {
                                    mlat_hub.note_adsb(update.icao24, now);
                                }

                                metrics.observe_message();
                                metrics.decode_latency.observe(t0.elapsed().as_secs_f64());

                                if let (Some(lat), Some(lon)) = (update.latitude, update.longitude)
                                {
                                    let r = range_nm(receiver_lat, receiver_lon, lat, lon);
                                    if r > metrics.max_range_nm.get() {
                                        metrics.max_range_nm.set(r);
                                    }
                                }

                                let mut store_w = store.write().await;
                                store_w.apply(update);
                                metrics.aircraft_current.set(store_w.len() as i64);
                            }
                            Ok(None) => {}
                            Err(e) => {
                                metrics.decode_errors.inc();
                                tracing::debug!(
                                    error = %e,
                                    source = %ingest.source,
                                    "decode failed"
                                );
                            }
                        }
                    }
                }
            }
        });
    }

    // Periodic: expire, snapshot, metrics series, WS delta broadcast
    {
        let store = store.clone();
        let history = history.clone();
        let metrics = metrics.clone();
        let state = state.clone();
        let snap_every = args.snapshot_interval;
        let cancel = shutdown.clone();
        tasks.spawn(async move {
            let mut tick: u64 = 0;
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            let mut prev_rev: std::collections::HashMap<Icao24, (u64, u64)> =
                std::collections::HashMap::new();
            loop {
                tokio::select! {
                    () = cancel.cancelled() => break,
                    _ = interval.tick() => {}
                }
                tick += 1;
                let now = now_secs();

                {
                    let mut s = store.write().await;
                    s.expire(now);
                    metrics.aircraft_current.set(s.len() as i64);
                }

                metrics.tick_rate();
                let summary = metrics.summary();
                {
                    let mut rs = state.rate_series.write().await;
                    rs.push(now, summary.message_rate);
                    let mut aseries = state.aircraft_series.write().await;
                    aseries.push(now, summary.aircraft_current as f64);
                    let mut range = state.range_series.write().await;
                    range.push(now, summary.max_range_nm);
                }

                let mut aircraft = store.read().await.list_owned();

                let need_history = tick.is_multiple_of(snap_every);
                if tick.is_multiple_of(60) {
                    prev_rev = aircraft
                        .iter()
                        .map(|a| (a.icao24, (a.messages, a.last_seen.to_bits())))
                        .collect();
                    let snapshot = if need_history {
                        aircraft.clone()
                    } else {
                        std::mem::take(&mut aircraft)
                    };
                    let _ = state.ws_tx.send(WsMessage::Snapshot {
                        aircraft: snapshot,
                    });
                } else {
                    let mut upsert = Vec::new();
                    let mut seen = std::collections::HashSet::new();
                    for a in &aircraft {
                        seen.insert(a.icao24);
                        let rev = (a.messages, a.last_seen.to_bits());
                        if prev_rev.get(&a.icao24) != Some(&rev) {
                            prev_rev.insert(a.icao24, rev);
                            upsert.push(a.clone());
                        }
                    }
                    let remove: Vec<Icao24> = prev_rev
                        .keys()
                        .filter(|icao| !seen.contains(*icao))
                        .copied()
                        .collect();
                    for icao in &remove {
                        prev_rev.remove(icao);
                    }
                    if !upsert.is_empty() || !remove.is_empty() {
                        let _ = state.ws_tx.send(WsMessage::Delta { upsert, remove });
                    }
                }

                let _ = state.ws_tx.send(WsMessage::Stats {
                    messages_total: summary.messages_total,
                    aircraft_current: summary.aircraft_current,
                    message_rate: summary.message_rate,
                    max_range_nm: summary.max_range_nm,
                });

                if need_history {
                    if let Err(e) = history.record_snapshot(&aircraft, now).await {
                        warn!(error = %e, "snapshot failed");
                    }
                    let _ = history.prune(now - 48.0 * 3600.0).await;
                }
            }
        });
    }

    let app = build_router(state);
    info!(%args.listen, "HTTP listening (UI embedded)");
    let listener = tokio::net::TcpListener::bind(args.listen).await?;
    let cancel = shutdown.clone();
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            info!("shutdown signal received");
            cancel.cancel();
        })
        .await?;

    shutdown.cancel();
    while tasks.join_next().await.is_some() {}
    info!("shutdown complete");
    Ok(())
}

pub(crate) struct IngestFrame {
    pub frame: crate::beast::BeastFrame,
    pub message_source: MessageSource,
    pub is_demo: bool,
    pub source: std::sync::Arc<str>,
}

fn parse_sources(raw: &[String]) -> Vec<SourceConfig> {
    raw.iter()
        .filter_map(|s| {
            // name=host:port or name=host:port:mlat
            let (name, rest) = s.split_once('=')?;
            let parts: Vec<&str> = rest.split(':').collect();
            if parts.len() < 2 {
                return None;
            }
            let host = parts.first()?.to_string();
            let port: u16 = parts.get(1)?.parse().ok()?;
            let is_mlat = parts.get(2).is_some_and(|p| *p == "mlat");
            Some(SourceConfig {
                name: name.to_string(),
                host,
                port,
                is_mlat,
            })
        })
        .collect()
}

async fn seed_sample_db(history: &HistoryStore) -> Result<()> {
    use crate::state::AircraftDbEntry;

    let samples = [
        (
            "3c66e6",
            AircraftDbEntry {
                registration: Some("D-AIMA".into()),
                typecode: Some("A320".into()),
                type_long: Some("AIRBUS A-320".into()),
                year: None,
                operator: Some("Lufthansa".into()),
                db_flags: 0,
            },
        ),
        (
            "4b1804",
            AircraftDbEntry {
                registration: Some("HB-JXA".into()),
                typecode: Some("BCS3".into()),
                type_long: Some("AIRBUS A-220-300".into()),
                year: None,
                operator: Some("Swiss".into()),
                db_flags: 0,
            },
        ),
        (
            "a37c5a",
            AircraftDbEntry {
                registration: Some("N326SB".into()),
                typecode: Some("C172".into()),
                type_long: Some("CESSNA 172 Skyhawk".into()),
                year: None,
                operator: None,
                db_flags: 0,
            },
        ),
    ];
    for (icao, entry) in samples {
        history.upsert_aircraft_db(icao, &entry).await?;
    }
    Ok(())
}
