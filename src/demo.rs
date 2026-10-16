//! Synthetic aircraft generator for UI development without a Beast feed.

use std::f64::consts::PI;

use crate::beast::BeastFrame;
use crate::decode::MessageSource;
use crate::icao::Icao24;
use rand::{Rng, SeedableRng};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::IngestFrame;

struct DemoPlane {
    icao: [u8; 3],
    callsign: &'static str,
    lat: f64,
    lon: f64,
    alt: i32,
    track: f64,
    gs: f64,
    phase: f64,
}

pub fn spawn(
    tx: mpsc::Sender<IngestFrame>,
    center_lat: f64,
    center_lon: f64,
    cancel: CancellationToken,
) {
    tokio::spawn(async move {
        info!("demo mode: generating synthetic aircraft");
        let mut planes = vec![
            DemoPlane {
                icao: [0x3c, 0x66, 0xe6],
                callsign: "DLH4AK",
                lat: center_lat + 0.15,
                lon: center_lon - 0.1,
                alt: 35000,
                track: 90.0,
                gs: 450.0,
                phase: 0.0,
            },
            DemoPlane {
                icao: [0x4b, 0x18, 0x04],
                callsign: "SWR12T",
                lat: center_lat - 0.08,
                lon: center_lon + 0.12,
                alt: 28000,
                track: 220.0,
                gs: 410.0,
                phase: 1.2,
            },
            DemoPlane {
                icao: [0xa3, 0x7c, 0x5a],
                callsign: "N326SB",
                lat: center_lat + 0.02,
                lon: center_lon + 0.03,
                alt: 4500,
                track: 45.0,
                gs: 120.0,
                phase: 2.5,
            },
            DemoPlane {
                icao: [0x48, 0x40, 0xb2],
                callsign: "KLM123",
                lat: center_lat - 0.2,
                lon: center_lon - 0.15,
                alt: 39000,
                track: 10.0,
                gs: 480.0,
                phase: 3.8,
            },
            DemoPlane {
                icao: [0x39, 0x6c, 0x00],
                callsign: "AFR45E",
                lat: center_lat + 0.25,
                lon: center_lon + 0.2,
                alt: 32000,
                track: 270.0,
                gs: 430.0,
                phase: 5.0,
            },
        ];

        let mut tick = interval(Duration::from_millis(500));
        let mut rng = rand::rngs::StdRng::from_os_rng();

        loop {
            tokio::select! {
                () = cancel.cancelled() => return,
                _ = tick.tick() => {}
            }
            for p in &mut planes {
                p.phase += 0.02;
                let rad = p.track.to_radians();
                // nm per half-second roughly from gs (knots)
                let nm = p.gs / 3600.0 * 0.5;
                let dlat = (nm / 60.0) * rad.cos();
                let dlon = (nm / 60.0) * rad.sin() / center_lat.to_radians().cos();
                p.lat += dlat;
                p.lon += dlon;
                p.track = (p.track
                    + rng.random_range(-1.0..1.0)
                    + 0.3 * (p.phase * 0.1).sin())
                    .rem_euclid(360.0);
                p.alt += rng.random_range(-50..50);
                p.alt = p.alt.clamp(1000, 42000);

                // Emit identity + position-like updates via synthetic Mode-S-ish payloads.
                // We bypass Mode-S CRC by feeding the decoder through a side channel:
                // craft Beast frames with valid-looking DF17 if possible; otherwise use
                // direct state injection via a minimal DF11-like all-call isn't enough.
                // Instead we inject through a parallel path using empty Mode-S long frames
                // and let demo apply updates in the companion task below — actually we
                // push special demo frames that the main loop won't decode, so we inject
                // DecodedUpdate-style data by packing lat/lon into a custom channel.
                //
                // Practical approach: send frames that fail decode, AND also send
                // parallel synthetic updates via embedding in signal/mlat fields.
                // Cleaner: use the demo inject path with fabricated BeastFrame +
                // a separate update sender. For simplicity we send Mode-S long zeros
                // (will count as decode errors) and rely on inject_updates below.

                let _ = (p.callsign, rad, PI);
            }

            // Direct state injection path: convert demo planes to synthetic "decoded"
            // by encoding a tiny custom Beast Mode-AC frame as heartbeat, while the
            // companion inject happens via channel of fabricated frames with payload
            // that we interpret in demo-aware mode.
            //
            // Simplest robust approach used here: send IngestFrame with Mode-S short
            // all-zero payload (skipped by decoder) is useless. Instead patch: use
            // mpsc to send fake frames AND have demo also write via a shared store.
            //
            // We'll encode aircraft state into Beast Mode-AC (ignored) and use a
            // side channel — actually the cleanest for this MVP is to push
            // fabricated Mode-S DF17 hex from known good samples rotated with
            // CPR. Even simpler: inject via `demo_inject` message type on the same
            // channel by putting lat/lon in mlat_timestamp bits and icao in payload.
            for p in &planes {
                let payload = build_demo_payload(p);

                let frame = BeastFrame {
                    msg_type: crate::beast::BeastMessageType::ModeSLong,
                    mlat_timestamp: encode_callsign_track(p.callsign, p.track, p.gs),
                    signal_level: 180,
                    payload,
                };

                if tx
                    .send(IngestFrame {
                        frame,
                        message_source: MessageSource::Adsb,
                        is_demo: true,
                        source: std::sync::Arc::from("demo"),
                    })
                    .await
                    .is_err()
                {
                    return;
                }
            }
        }
    });
}

fn build_demo_payload(p: &DemoPlane) -> Vec<u8> {
    let mut payload = vec![0u8; 14];
    // DF17 header-ish: 0x8D + ICAO — not CRC-valid, but we handle demo
    // specially: message_source Other + payload laid out for demo parser.
    if let Some(b) = payload.get_mut(0) {
        *b = 0x8d;
    }
    for (i, byte) in p.icao.iter().enumerate() {
        if let Some(slot) = payload.get_mut(i + 1) {
            *slot = *byte;
        }
    }
    let alt_enc = (p.alt / 25) as u16;
    if let Some(b) = payload.get_mut(4) {
        *b = (alt_enc >> 8) as u8;
    }
    if let Some(b) = payload.get_mut(5) {
        *b = alt_enc as u8;
    }
    let lat_i = (p.lat * 1_000_000.0) as i32;
    let lon_i = (p.lon * 1_000_000.0) as i32;
    if let Some(lat_slot) = payload.get_mut(6..10).and_then(<[u8]>::as_mut_array) {
        *lat_slot = lat_i.to_be_bytes();
    }
    if let Some(lon_slot) = payload.get_mut(10..14).and_then(<[u8]>::as_mut_array) {
        *lon_slot = lon_i.to_be_bytes();
    }
    payload
}

fn encode_callsign_track(callsign: &str, track: f64, gs: f64) -> u64 {
    // Pack into 48-bit mlat field: [track_u16][gs_u16][cs_hash_u16]
    let t = (track * 10.0) as u16;
    let g = gs as u16;
    let mut h: u16 = 0;
    for b in callsign.bytes() {
        h = h.wrapping_mul(31).wrapping_add(u16::from(b));
    }
    ((u64::from(t) << 32) | (u64::from(g) << 16) | u64::from(h)) & 0xffff_ffff_ffff
}

/// Parse demo payload into a DecodedUpdate. Called from main when source is demo.
pub fn try_parse_demo(frame: &BeastFrame) -> Option<crate::decode::DecodedUpdate> {
    let payload = frame.mode_s_payload()?;
    if payload.len() != 14 || payload.first().copied() != Some(0x8d) {
        return None;
    }
    let icao24 = Icao24::from_bytes(*payload.get(1..4)?.as_array()?);
    let alt = (u16::from(*payload.get(4)?) << 8 | u16::from(*payload.get(5)?)) as i32 * 25;
    let lat = i32::from_be_bytes(*payload.get(6..10)?.as_array()?) as f64 / 1_000_000.0;
    let lon = i32::from_be_bytes(*payload.get(10..14)?.as_array()?) as f64 / 1_000_000.0;
    let packed = frame.mlat_timestamp;
    let track = ((packed >> 32) & 0xffff) as f64 / 10.0;
    let gs = ((packed >> 16) & 0xffff) as f64;
    let callsign = demo_callsign_for(icao24);

    Some(crate::decode::DecodedUpdate {
        callsign: Some(callsign),
        squawk: Some("1000".into()),
        altitude: Some(alt),
        altitude_baro: Some(alt),
        latitude: Some(lat),
        longitude: Some(lon),
        groundspeed: Some(gs),
        track: Some(track),
        vertical_rate: Some(0),
        heading: Some(track),
        nac_v: Some(2),
        nac_p: Some(9),
        nuc_p: Some(7),
        sil: Some(3),
        adsb_version: Some(2),
        category: Some("A3".into()),
        emergency: Some("none".into()),
        spi: Some(false),
        on_ground: Some(false),
        source: MessageSource::Adsb,
        ..crate::decode::DecodedUpdate::bare(
            icao24,
            crate::state::now_secs(),
            Some(frame.signal_level),
        )
    })
}

fn demo_callsign_for(icao: Icao24) -> String {
    match icao.to_hex().as_str() {
        "3c66e6" => "DLH4AK".into(),
        "4b1804" => "SWR12T".into(),
        "a37c5a" => "N326SB".into(),
        "4840b2" => "KLM123".into(),
        "396c00" => "AFR45E".into(),
        hex => format!("DEMO{}", &hex[hex.len().saturating_sub(3)..]),
    }
}
