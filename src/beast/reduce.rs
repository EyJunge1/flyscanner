//! Beast-Reduce rate limiting (readsb-compatible field intervals).
//!
//! Framing stays normal Beast; only which Mode-S frames are forwarded changes.
//! Interval classes mirror readsb `track.c` accept_data reduce multipliers.

use std::collections::HashMap;

use crate::icao::Icao24;

/// How aggressively to forward a Mode-S frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReduceClass {
    /// Position fixes — most frequent (`≈ 0.375 × I`).
    Position,
    /// Velocity / altitude / CPR — often (`≈ 0.875 × I`).
    Often,
    /// Ident / squawk / status / DF11 — rare (`≈ 4 × I`).
    Rare,
    /// Drop (no useful state for aggregators).
    Drop,
}

/// Outbound feed protocol variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedMode {
    /// Full-rate Beast.
    Beast,
    /// Rate-limited Beast (no UUID handshake).
    BeastReduce,
    /// Rate-limited Beast + UUID handshake on connect.
    BeastReducePlus,
}

impl FeedMode {
    #[must_use]
    pub fn is_reduce(self) -> bool {
        matches!(self, Self::BeastReduce | Self::BeastReducePlus)
    }

    #[must_use]
    pub fn needs_uuid_handshake(self) -> bool {
        matches!(self, Self::BeastReducePlus)
    }
}

/// Per-ICAO last-forward timestamps for each reduce class.
#[derive(Debug, Default)]
pub struct BeastReducer {
    /// Base interval `I` in seconds (Ultrafeeder default 0.5).
    interval: f64,
    /// icao24 → (last_position, last_often, last_rare)
    last: HashMap<Icao24, [f64; 3]>,
}

impl BeastReducer {
    #[must_use]
    pub fn new(interval_secs: f64) -> Self {
        Self {
            interval: interval_secs.clamp(0.05, 14.999),
            last: HashMap::new(),
        }
    }

    /// Returns true if this Mode-S payload should be forwarded now.
    pub fn accept(&mut self, payload: &[u8], now: f64) -> bool {
        let class = classify_payload(payload);
        if matches!(class, ReduceClass::Drop) {
            return false;
        }
        let Some(icao) = icao_from_payload(payload) else {
            return false;
        };
        let idx = match class {
            ReduceClass::Position => 0usize,
            ReduceClass::Often => 1,
            ReduceClass::Rare => 2,
            ReduceClass::Drop => return false,
        };
        let min_gap = match class {
            ReduceClass::Position => self.interval * 0.375,
            ReduceClass::Often => self.interval * 0.875,
            ReduceClass::Rare => (self.interval * 4.0).min(7.0),
            ReduceClass::Drop => return false,
        };
        let entry = self.last.entry(icao).or_insert([0.0, 0.0, 0.0]);
        let Some(last_ts) = entry.get(idx).copied() else {
            return false;
        };
        if now - last_ts < min_gap {
            return false;
        }
        if let Some(slot) = entry.get_mut(idx) {
            *slot = now;
        }
        true
    }

    /// Drop stale ICAO entries (memory hygiene).
    pub fn expire(&mut self, now: f64, ttl_secs: f64) {
        self.last.retain(|_, times| {
            let newest = times.iter().copied().fold(0.0_f64, f64::max);
            now - newest <= ttl_secs
        });
    }
}

/// Classify Mode-S payload for reduce forwarding.
#[must_use]
pub fn classify_payload(payload: &[u8]) -> ReduceClass {
    let Some(&b0) = payload.first() else {
        return ReduceClass::Drop;
    };
    if payload.len() < 7 {
        return ReduceClass::Drop;
    }
    let df = b0 >> 3;
    match df {
        11 => ReduceClass::Rare,
        0 | 4 | 5 => ReduceClass::Often,
        16 | 20 | 21 => ReduceClass::Often,
        17 | 18 => {
            let Some(&b4) = payload.get(4) else {
                return ReduceClass::Drop;
            };
            let tc = b4 >> 3;
            match tc {
                1..=4 => ReduceClass::Rare,
                5..=8 => ReduceClass::Position,
                9..=18 | 20..=22 => ReduceClass::Position,
                19 => ReduceClass::Often,
                28 => ReduceClass::Rare,
                29 | 31 => ReduceClass::Rare,
                _ => ReduceClass::Drop,
            }
        }
        _ => ReduceClass::Drop,
    }
}

/// Extract ICAO24 hex from common Mode-S DFs.
#[must_use]
pub fn icao_from_payload(payload: &[u8]) -> Option<Icao24> {
    let b0 = *payload.first()?;
    let df = b0 >> 3;
    match df {
        11 | 17 | 18 | 0 | 4 | 5 | 16 | 20 | 21 => {
            Some(Icao24::from_bytes(*payload.get(1..4)?.as_array()?))
        }
        _ => None,
    }
}

/// Build beast_reduce_plus connect handshake: `1A E4` + UUID + `1A WO` + 5 heartbeats.
#[must_use]
pub fn reduce_plus_handshake(uuid: &str) -> Vec<u8> {
    let mut padded = uuid.trim().to_string();
    if padded.len() > 36 {
        padded.truncate(36);
    }
    while padded.len() < 36 {
        padded.push('f');
    }

    let mut out = Vec::with_capacity(64);
    out.push(0x1a);
    out.push(0xe4);
    out.extend_from_slice(padded.as_bytes());
    out.push(0x1a);
    out.push(b'W');
    out.push(b'O');
    for _ in 0..5 {
        out.extend_from_slice(&[
            0x1a, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);
    }
    out
}

/// Normalize / generate a 36-char UUID string for Plus feeds.
#[must_use]
pub fn normalize_uuid(raw: Option<&str>) -> String {
    match raw.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) if s.len() == 36 => s.to_string(),
        Some(s)
            if let hex = s
                .chars()
                .filter(char::is_ascii_hexdigit)
                .collect::<String>()
                && let (Some(a), Some(b), Some(c), Some(d), Some(e)) = (
                    hex.get(0..8),
                    hex.get(8..12),
                    hex.get(12..16),
                    hex.get(16..20),
                    hex.get(20..32),
                ) =>
        {
            format!("{a}-{b}-{c}-{d}-{e}")
        }
        _ => uuid::Uuid::new_v4().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches;

    #[allow(clippy::indexing_slicing, clippy::expect_used)]
    fn set(v: &mut [u8], i: usize, b: u8) {
        v[i] = b;
    }

    #[test]
    fn classifies_adsb_position_and_ident() {
        let mut pos = vec![0u8; 14];
        set(&mut pos, 0, 17 << 3);
        set(&mut pos, 4, 11 << 3);
        assert_matches!(classify_payload(&pos), ReduceClass::Position);

        let mut ident = vec![0u8; 14];
        set(&mut ident, 0, 17 << 3);
        set(&mut ident, 4, 4 << 3);
        assert_matches!(classify_payload(&ident), ReduceClass::Rare);

        let mut vel = vec![0u8; 14];
        set(&mut vel, 0, 17 << 3);
        set(&mut vel, 4, 19 << 3);
        assert_matches!(classify_payload(&vel), ReduceClass::Often);
    }

    #[test]
    fn reducer_rate_limits_position() {
        let mut r = BeastReducer::new(0.5);
        let mut pos = vec![0u8; 14];
        set(&mut pos, 0, 17 << 3);
        set(&mut pos, 1, 0xab);
        set(&mut pos, 2, 0xcd);
        set(&mut pos, 3, 0xef);
        set(&mut pos, 4, 11 << 3);
        assert!(r.accept(&pos, 1000.0));
        assert!(!r.accept(&pos, 1000.05));
        assert!(r.accept(&pos, 1000.2));
    }

    #[test]
    fn handshake_layout() {
        let hs = reduce_plus_handshake("12345678-1234-1234-1234-123456789abc");
        assert_eq!(hs.first().copied(), Some(0x1a));
        assert_eq!(hs.get(1).copied(), Some(0xe4));
        assert_eq!(
            hs.get(2..38)
                .map(|s| s == b"12345678-1234-1234-1234-123456789abc"),
            Some(true)
        );
        assert_eq!(hs.get(38).copied(), Some(0x1a));
        assert_eq!(hs.get(39).copied(), Some(b'W'));
        assert_eq!(hs.get(40).copied(), Some(b'O'));
        assert_eq!(hs.len(), 41 + 5 * 11);
    }

    #[test]
    fn normalize_formats_hex() {
        let u = normalize_uuid(Some("aabbccddeeff00112233445566778899"));
        assert_eq!(u, "aabbccdd-eeff-0011-2233-445566778899");
    }
}
