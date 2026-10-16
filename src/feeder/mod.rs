//! Multi-source Beast ingest with simple deduplication and MLAT tagging.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use crate::beast::{BeastClient, BeastFrame, BeastSource};
use crate::decode::MessageSource;
use crate::icao::Icao24;
use crate::metrics::Metrics;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[derive(Debug, Clone)]
pub struct SourceConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub is_mlat: bool,
}

#[derive(Debug, Clone)]
pub struct TaggedFrame {
    pub source: Arc<str>,
    pub frame: BeastFrame,
    pub message_source: MessageSource,
}

/// Deduplicate identical Mode-S payloads within a short window.
///
/// Keys are payload hashes (not owned `Vec<u8>`) to avoid per-frame allocations.
pub struct Deduper {
    seen: HashMap<u64, f64>,
    window_secs: f64,
    since_retain: u32,
}

impl Deduper {
    pub fn new(window_ms: u64) -> Self {
        Self {
            seen: HashMap::new(),
            window_secs: window_ms as f64 / 1000.0,
            since_retain: 0,
        }
    }

    fn payload_hash(payload: &[u8]) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        payload.hash(&mut hasher);
        hasher.finish()
    }

    /// Returns true if the payload is new (should be processed).
    pub fn accept(&mut self, payload: &[u8], now: f64) -> bool {
        // Amortize expiry: full retain every 64 accepts instead of every frame.
        self.since_retain += 1;
        if self.since_retain >= 64 {
            self.since_retain = 0;
            let window = self.window_secs;
            self.seen.retain(|_, ts| now - *ts <= window);
        }

        let key = Self::payload_hash(payload);
        if let Some(ts) = self.seen.get(&key)
            && now - *ts <= self.window_secs
        {
            return false;
        }
        self.seen.insert(key, now);
        true
    }
}

/// Spawn Beast clients for multiple sources; merge into one channel.
pub fn spawn_sources(
    sources: Vec<SourceConfig>,
    metrics: Option<Arc<Metrics>>,
    cancel: CancellationToken,
) -> mpsc::Receiver<TaggedFrame> {
    let (tx, rx) = mpsc::channel(4096);

    for src in sources {
        let (frame_tx, mut frame_rx) = mpsc::channel(1024);
        BeastClient::spawn(
            BeastSource::new(src.host.clone(), src.port),
            frame_tx,
            metrics.clone(),
            cancel.clone(),
        );
        let out = tx.clone();
        let name: Arc<str> = Arc::from(src.name);
        let is_mlat = src.is_mlat;
        let cancel = cancel.clone();
        tokio::spawn(async move {
            info!(source = %name, is_mlat, "feeder source started");
            loop {
                tokio::select! {
                    () = cancel.cancelled() => break,
                    frame = frame_rx.recv() => {
                        let Some(frame) = frame else { break };
                        let message_source = if is_mlat {
                            MessageSource::Mlat
                        } else {
                            MessageSource::Adsb
                        };
                        if out
                            .send(TaggedFrame {
                                source: Arc::clone(&name),
                                frame,
                                message_source,
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

    drop(tx);
    rx
}

/// MLAT hub: merge MLAT-tagged updates preferring ADS-B position when fresher.
#[derive(Debug, Default)]
pub struct MlatHub {
    /// last ADS-B position timestamp per ICAO
    last_adsb: HashMap<Icao24, f64>,
}

impl MlatHub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn note_adsb(&mut self, icao: Icao24, ts: f64) {
        self.last_adsb.insert(icao, ts);
    }

    /// Prefer MLAT only if no ADS-B position in the last `fresh_secs`.
    pub fn prefer_mlat(&self, icao: Icao24, now: f64, fresh_secs: f64) -> bool {
        match self.last_adsb.get(&icao) {
            Some(ts) if now - *ts > fresh_secs => true,
            Some(_) => false,
            None => true,
        }
    }
}

#[expect(dead_code)]
pub fn default_reconnect_backoff() -> Duration {
    Duration::from_secs(2)
}
