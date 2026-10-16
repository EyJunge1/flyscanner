//! Aircraft state store — Structure-of-Arrays (SoA) layout with AoS DTO export.
//!
//! Hot paths update columnar fields by index; JSON/WS still see classic `Aircraft`
//! structs assembled on demand (serde-friendly, frontend-compatible).

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::decode::{DecodedUpdate, MessageSource};
use crate::icao::Icao24;
use crate::metrics::range_nm;
use serde::{Deserialize, Serialize};

/// Max implied groundspeed (kt) before a position update is rejected as outlier.
const MAX_IMPLIED_SPEED_KT: f64 = 1200.0;
/// Always accept a new fix after this gap (aircraft reappears / CPR reacquire).
const SPEED_CHECK_GAP_SECS: f64 = 60.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackPoint {
    pub ts: f64,
    pub lat: f64,
    pub lon: f64,
    pub alt: Option<i32>,
}

/// Array-of-Structs DTO for API / WS / history JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aircraft {
    pub icao24: Icao24,
    pub first_seen: f64,
    pub last_seen: f64,
    pub callsign: Option<String>,
    pub squawk: Option<String>,
    pub altitude: Option<i32>,
    pub altitude_baro: Option<i32>,
    pub altitude_geom: Option<i32>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub groundspeed: Option<f64>,
    pub track: Option<f64>,
    pub vertical_rate: Option<i16>,
    pub heading: Option<f64>,
    pub ias: Option<u16>,
    pub tas: Option<u16>,
    pub nac_v: Option<u8>,
    pub nac_p: Option<u8>,
    pub nuc_p: Option<u8>,
    pub sil: Option<u8>,
    pub adsb_version: Option<u8>,
    pub geo_minus_baro: Option<i16>,
    pub category: Option<String>,
    pub emergency: Option<String>,
    pub spi: Option<bool>,
    pub on_ground: Option<bool>,
    pub rssi: Option<f64>,
    pub messages: u64,
    pub source: MessageSource,
    pub registration: Option<String>,
    pub typecode: Option<String>,
    pub type_long: Option<String>,
    pub year: Option<String>,
    pub operator: Option<String>,
    /// tar1090-db bitfield: military=1, interesting=2, PIA=4, LADD=8
    pub db_flags: u8,
    pub track_history: Vec<TrackPoint>,
}

fn source_rank(src: MessageSource) -> u8 {
    match src {
        MessageSource::Adsb => 4,
        MessageSource::Mlat => 3,
        MessageSource::Tisb => 2,
        MessageSource::ModeS => 1,
        MessageSource::Other => 0,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AircraftDbEntry {
    pub registration: Option<String>,
    pub typecode: Option<String>,
    pub type_long: Option<String>,
    pub year: Option<String>,
    pub operator: Option<String>,
    pub db_flags: u8,
}

/// Columnar aircraft store (SoA).
#[derive(Debug, Default)]
pub struct AircraftStore {
    index: HashMap<Icao24, usize>,
    // identity / timing
    icao: Vec<Icao24>,
    first_seen: Vec<f64>,
    last_seen: Vec<f64>,
    messages: Vec<u64>,
    source: Vec<MessageSource>,
    // position / kinematics (hot)
    latitude: Vec<Option<f64>>,
    longitude: Vec<Option<f64>>,
    altitude: Vec<Option<i32>>,
    altitude_baro: Vec<Option<i32>>,
    altitude_geom: Vec<Option<i32>>,
    groundspeed: Vec<Option<f64>>,
    track: Vec<Option<f64>>,
    vertical_rate: Vec<Option<i16>>,
    heading: Vec<Option<f64>>,
    pos_ts: Vec<Option<f64>>,
    ias: Vec<Option<u16>>,
    tas: Vec<Option<u16>>,
    // quality / status
    nac_v: Vec<Option<u8>>,
    nac_p: Vec<Option<u8>>,
    nuc_p: Vec<Option<u8>>,
    sil: Vec<Option<u8>>,
    adsb_version: Vec<Option<u8>>,
    geo_minus_baro: Vec<Option<i16>>,
    spi: Vec<Option<bool>>,
    on_ground: Vec<Option<bool>>,
    rssi: Vec<Option<f64>>,
    // cold string fields
    callsign: Vec<Option<String>>,
    squawk: Vec<Option<String>>,
    category: Vec<Option<String>>,
    emergency: Vec<Option<String>>,
    registration: Vec<Option<String>>,
    typecode: Vec<Option<String>>,
    type_long: Vec<Option<String>>,
    year: Vec<Option<String>>,
    operator: Vec<Option<String>>,
    db_flags: Vec<u8>,
    track_history: Vec<Vec<TrackPoint>>,
    db: HashMap<Icao24, AircraftDbEntry>,
    pub ttl: Duration,
    pub max_track_points: usize,
}

impl AircraftStore {
    pub fn new(ttl_secs: u64, max_track_points: usize) -> Self {
        Self {
            ttl: Duration::from_secs(ttl_secs),
            max_track_points,
            ..Self::default()
        }
    }

    pub fn load_db(&mut self, entries: HashMap<Icao24, AircraftDbEntry>) {
        self.db = entries;
    }

    #[expect(dead_code)]
    pub fn upsert_db(&mut self, icao: Icao24, entry: AircraftDbEntry) {
        self.db.insert(icao, entry);
    }

    pub fn apply(&mut self, update: DecodedUpdate) {
        let icao = update.icao24;
        let idx = if let Some(&i) = self.index.get(&icao) {
            i
        } else {
            self.push_new(icao, update.timestamp)
        };
        self.apply_at(idx, &update);
    }

    fn push_new(&mut self, icao: Icao24, ts: f64) -> usize {
        let i = self.icao.len();
        self.index.insert(icao, i);
        self.icao.push(icao);
        self.first_seen.push(ts);
        self.last_seen.push(ts);
        self.messages.push(0);
        self.source.push(MessageSource::Other);
        self.latitude.push(None);
        self.longitude.push(None);
        self.altitude.push(None);
        self.altitude_baro.push(None);
        self.altitude_geom.push(None);
        self.groundspeed.push(None);
        self.track.push(None);
        self.vertical_rate.push(None);
        self.heading.push(None);
        self.pos_ts.push(None);
        self.ias.push(None);
        self.tas.push(None);
        self.nac_v.push(None);
        self.nac_p.push(None);
        self.nuc_p.push(None);
        self.sil.push(None);
        self.adsb_version.push(None);
        self.geo_minus_baro.push(None);
        self.spi.push(None);
        self.on_ground.push(None);
        self.rssi.push(None);
        self.callsign.push(None);
        self.squawk.push(None);
        self.category.push(None);
        self.emergency.push(None);

        let (reg, tc, tl, yr, op, flags) = if let Some(db) = self.db.get(&icao) {
            (
                db.registration.clone(),
                db.typecode.clone(),
                db.type_long.clone(),
                db.year.clone(),
                db.operator.clone(),
                db.db_flags,
            )
        } else {
            let hex = icao.to_hex();
            let reg = rs1090::data::patterns::aircraft_information(&hex, None)
                .ok()
                .and_then(|info| info.registration);
            (reg, None, None, None, None, 0)
        };
        self.registration.push(reg);
        self.typecode.push(tc);
        self.type_long.push(tl);
        self.year.push(yr);
        self.operator.push(op);
        self.db_flags.push(flags);
        self.track_history.push(Vec::new());
        i
    }

    fn apply_at(&mut self, i: usize, update: &DecodedUpdate) {
        let Some(last_seen) = self.last_seen.get_mut(i) else {
            return;
        };
        *last_seen = update.timestamp;
        if let Some(m) = self.messages.get_mut(i) {
            *m += 1;
        }
        if let Some(src) = self.source.get_mut(i)
            && source_rank(update.source) >= source_rank(*src)
        {
            *src = update.source;
        }

        set_opt_clone(&mut self.callsign, i, update.callsign.as_ref());
        set_opt_clone(&mut self.squawk, i, update.squawk.as_ref());

        if let Some(alt) = update.altitude_baro {
            set_opt(&mut self.altitude_baro, i, Some(alt));
            set_opt(&mut self.altitude, i, Some(alt));
        }
        if let Some(alt) = update.altitude_geom {
            set_opt(&mut self.altitude_geom, i, Some(alt));
            if self.altitude_baro.get(i).copied().flatten().is_none() {
                set_opt(&mut self.altitude, i, Some(alt));
            }
        } else if let Some(alt) = update.altitude {
            set_opt(&mut self.altitude, i, Some(alt));
        }

        set_opt(&mut self.groundspeed, i, update.groundspeed);
        set_opt(&mut self.track, i, update.track);
        set_opt(&mut self.vertical_rate, i, update.vertical_rate);
        set_opt(&mut self.heading, i, update.heading);
        set_opt(&mut self.ias, i, update.ias);
        set_opt(&mut self.tas, i, update.tas);
        set_opt(&mut self.nac_v, i, update.nac_v);
        set_opt(&mut self.nac_p, i, update.nac_p);
        set_opt(&mut self.nuc_p, i, update.nuc_p);
        set_opt(&mut self.sil, i, update.sil);
        set_opt(&mut self.adsb_version, i, update.adsb_version);
        set_opt(&mut self.geo_minus_baro, i, update.geo_minus_baro);
        set_opt_clone(&mut self.category, i, update.category.as_ref());
        set_opt_clone(&mut self.emergency, i, update.emergency.as_ref());
        set_opt(&mut self.spi, i, update.spi);
        set_opt(&mut self.on_ground, i, update.on_ground);

        if let Some(sig) = update.signal_level {
            set_opt(&mut self.rssi, i, Some(-(f64::from(sig) / 2.0)));
        }

        let (Some(lat), Some(lon)) = (update.latitude, update.longitude) else {
            return;
        };
        if !self.accept_position(i, lat, lon, update.timestamp, update.groundspeed) {
            return;
        }
        let prev_lat = self.latitude.get(i).copied().flatten();
        let prev_lon = self.longitude.get(i).copied().flatten();
        let changed = prev_lat.is_none_or(|p| (p - lat).abs() > 1e-6)
            || prev_lon.is_none_or(|p| (p - lon).abs() > 1e-6);
        set_opt(&mut self.latitude, i, Some(lat));
        set_opt(&mut self.longitude, i, Some(lon));
        set_opt(&mut self.pos_ts, i, Some(update.timestamp));
        if changed {
            let alt = self.altitude.get(i).copied().flatten();
            if let Some(hist) = self.track_history.get_mut(i) {
                hist.push(TrackPoint {
                    ts: update.timestamp,
                    lat,
                    lon,
                    alt,
                });
                if hist.len() > self.max_track_points {
                    let drain = hist.len() - self.max_track_points;
                    hist.drain(0..drain);
                }
            }
        }
    }

    fn accept_position(
        &self,
        i: usize,
        lat: f64,
        lon: f64,
        ts: f64,
        reported_gs: Option<f64>,
    ) -> bool {
        let Some(prev_lat) = self.latitude.get(i).copied().flatten() else {
            return true;
        };
        let Some(prev_lon) = self.longitude.get(i).copied().flatten() else {
            return true;
        };
        let Some(prev_ts) = self.pos_ts.get(i).copied().flatten() else {
            return true;
        };
        let dt = ts - prev_ts;
        if dt <= 0.05 {
            let d_nm = range_nm(prev_lat, prev_lon, lat, lon);
            return d_nm < 0.05;
        }
        if dt >= SPEED_CHECK_GAP_SECS {
            return true;
        }
        let dist_nm = range_nm(prev_lat, prev_lon, lat, lon);
        let speed_kt = dist_nm / (dt / 3600.0);
        let stored_gs = self.groundspeed.get(i).copied().flatten();
        let mut max_kt = reported_gs.or(stored_gs).map_or(MAX_IMPLIED_SPEED_KT, |gs| {
            (gs * 1.5).clamp(100.0, MAX_IMPLIED_SPEED_KT)
        });
        if self.on_ground.get(i).copied().flatten() == Some(true) {
            max_kt = max_kt.min(250.0);
        }
        speed_kt <= max_kt
    }

    fn assemble(&self, i: usize) -> Option<Aircraft> {
        Some(Aircraft {
            icao24: *self.icao.get(i)?,
            first_seen: *self.first_seen.get(i)?,
            last_seen: *self.last_seen.get(i)?,
            callsign: self.callsign.get(i)?.clone(),
            squawk: self.squawk.get(i)?.clone(),
            altitude: *self.altitude.get(i)?,
            altitude_baro: *self.altitude_baro.get(i)?,
            altitude_geom: *self.altitude_geom.get(i)?,
            latitude: *self.latitude.get(i)?,
            longitude: *self.longitude.get(i)?,
            groundspeed: *self.groundspeed.get(i)?,
            track: *self.track.get(i)?,
            vertical_rate: *self.vertical_rate.get(i)?,
            heading: *self.heading.get(i)?,
            ias: *self.ias.get(i)?,
            tas: *self.tas.get(i)?,
            nac_v: *self.nac_v.get(i)?,
            nac_p: *self.nac_p.get(i)?,
            nuc_p: *self.nuc_p.get(i)?,
            sil: *self.sil.get(i)?,
            adsb_version: *self.adsb_version.get(i)?,
            geo_minus_baro: *self.geo_minus_baro.get(i)?,
            category: self.category.get(i)?.clone(),
            emergency: self.emergency.get(i)?.clone(),
            spi: *self.spi.get(i)?,
            on_ground: *self.on_ground.get(i)?,
            rssi: *self.rssi.get(i)?,
            messages: *self.messages.get(i)?,
            source: *self.source.get(i)?,
            registration: self.registration.get(i)?.clone(),
            typecode: self.typecode.get(i)?.clone(),
            type_long: self.type_long.get(i)?.clone(),
            year: self.year.get(i)?.clone(),
            operator: self.operator.get(i)?.clone(),
            db_flags: *self.db_flags.get(i)?,
            track_history: self.track_history.get(i)?.clone(),
        })
    }

    pub fn get(&self, icao: Icao24) -> Option<Aircraft> {
        let &i = self.index.get(&icao)?;
        self.assemble(i)
    }

    pub fn get_str(&self, icao: &str) -> Option<Aircraft> {
        self.get(Icao24::parse(icao)?)
    }

    pub fn list_owned(&self) -> Vec<Aircraft> {
        (0..self.icao.len()).filter_map(|i| self.assemble(i)).collect()
    }

    pub fn expire(&mut self, now: f64) -> usize {
        let ttl = self.ttl.as_secs_f64();
        let before = self.len();
        let mut i = 0;
        while i < self.icao.len() {
            let Some(&last) = self.last_seen.get(i) else {
                break;
            };
            if now - last > ttl {
                self.swap_remove(i);
            } else {
                i += 1;
            }
        }
        before - self.len()
    }

    fn swap_remove(&mut self, i: usize) {
        let Some(removed) = self.icao.get(i).copied() else {
            return;
        };
        let last = self.icao.len().saturating_sub(1);
        if i != last
            && let Some(&moved) = self.icao.get(last)
        {
            self.index.insert(moved, i);
        }
        self.index.remove(&removed);

        macro_rules! sr {
            ($field:ident) => {
                if i < self.$field.len() {
                    self.$field.swap_remove(i);
                }
            };
        }
        sr!(icao);
        sr!(first_seen);
        sr!(last_seen);
        sr!(messages);
        sr!(source);
        sr!(latitude);
        sr!(longitude);
        sr!(altitude);
        sr!(altitude_baro);
        sr!(altitude_geom);
        sr!(groundspeed);
        sr!(track);
        sr!(vertical_rate);
        sr!(heading);
        sr!(pos_ts);
        sr!(ias);
        sr!(tas);
        sr!(nac_v);
        sr!(nac_p);
        sr!(nuc_p);
        sr!(sil);
        sr!(adsb_version);
        sr!(geo_minus_baro);
        sr!(spi);
        sr!(on_ground);
        sr!(rssi);
        sr!(callsign);
        sr!(squawk);
        sr!(category);
        sr!(emergency);
        sr!(registration);
        sr!(typecode);
        sr!(type_long);
        sr!(year);
        sr!(operator);
        sr!(db_flags);
        sr!(track_history);
    }

    pub fn len(&self) -> usize {
        self.icao.len()
    }

    #[expect(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.icao.is_empty()
    }

    /// Heatmap points from all track histories.
    pub fn heatmap_points(&self, max: usize) -> Vec<TrackPoint> {
        self.track_history
            .iter()
            .flat_map(|h| h.iter().cloned())
            .take(max)
            .collect()
    }
}

fn set_opt<T: Copy>(col: &mut [Option<T>], i: usize, v: Option<T>) {
    if let (Some(slot), Some(val)) = (col.get_mut(i), v) {
        *slot = Some(val);
    }
}

fn set_opt_clone(col: &mut [Option<String>], i: usize, v: Option<&String>) {
    if let (Some(slot), Some(val)) = (col.get_mut(i), v) {
        *slot = Some(val.clone());
    }
}

pub fn now_secs() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn update_at(icao: Icao24, ts: f64, lat: f64, lon: f64, gs: Option<f64>) -> DecodedUpdate {
        DecodedUpdate {
            icao24: icao,
            timestamp: ts,
            signal_level: None,
            callsign: None,
            squawk: None,
            altitude: Some(35000),
            altitude_baro: Some(35000),
            altitude_geom: None,
            latitude: Some(lat),
            longitude: Some(lon),
            groundspeed: gs,
            track: None,
            vertical_rate: None,
            heading: None,
            ias: None,
            tas: None,
            nac_v: None,
            nac_p: None,
            nuc_p: None,
            sil: None,
            adsb_version: None,
            geo_minus_baro: None,
            category: None,
            emergency: None,
            spi: None,
            on_ground: Some(false),
            source: MessageSource::Adsb,
        }
    }

    #[test]
    fn rejects_impossible_position_jump() {
        let icao = Icao24::from_bytes([0xab, 0xc1, 0x23]);
        let mut store = AircraftStore::new(60, 100);
        store.apply(update_at(icao, 1000.0, 52.0, 13.0, Some(450.0)));
        store.apply(update_at(icao, 1001.0, 60.0, 13.0, Some(450.0)));
        let Some(ac) = store.get(icao) else {
            return;
        };
        assert!((ac.latitude.unwrap_or_default() - 52.0).abs() < 1e-6);
        assert_eq!(ac.messages, 2);
    }

    #[test]
    fn accepts_plausible_position_move() {
        let icao = Icao24::from_bytes([0xab, 0xc1, 0x23]);
        let mut store = AircraftStore::new(60, 100);
        store.apply(update_at(icao, 1000.0, 52.0, 13.0, Some(450.0)));
        store.apply(update_at(icao, 1001.0, 52.00208, 13.0, Some(450.0)));
        let Some(ac) = store.get(icao) else {
            return;
        };
        assert!((ac.latitude.unwrap_or_default() - 52.00208).abs() < 1e-6);
    }

    #[test]
    fn db_flags_bits() {
        let flags: u8 = 1 | 2 | 8;
        assert_eq!(flags & 1, 1);
        assert_eq!(flags & 2, 2);
        assert_eq!(flags & 4, 0);
        assert_eq!(flags & 8, 8);
        assert_eq!(flags, 11);
    }

    #[test]
    fn soa_expire_compacts() {
        let mut store = AircraftStore::new(10, 100);
        store.apply(update_at(
            Icao24::from_bytes([0xab, 0xc1, 0x23]),
            1000.0,
            52.0,
            13.0,
            Some(450.0),
        ));
        store.apply(update_at(
            Icao24::from_bytes([0xde, 0xf4, 0x56]),
            1000.0,
            53.0,
            14.0,
            Some(400.0),
        ));
        assert_eq!(store.len(), 2);
        store.expire(1020.0);
        assert_eq!(store.len(), 0);
    }
}
