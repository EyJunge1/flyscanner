//! Local Turso (embedded libSQL) aircraft history — no Turso Cloud.
//! Schema via ordered `PRAGMA user_version` migrations (opencode-mem style).

mod migrations;

use crate::icao::Icao24;
use crate::state::{Aircraft, AircraftDbEntry, TrackPoint};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::info;
use turso::{Builder, Connection, Value};

use migrations::{apply_schema_migrations, HISTORY_MIGRATIONS};

#[derive(Clone)]
pub struct HistoryStore {
    conn: Connection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySnapshot {
    pub ts: f64,
    pub aircraft: Vec<Aircraft>,
}

impl HistoryStore {
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let path_str = path.as_ref().to_string_lossy().to_string();
        let db = Builder::new_local(&path_str).build().await?;
        let conn = db.connect()?;

        let version = apply_schema_migrations(&conn, HISTORY_MIGRATIONS).await?;
        info!(path = %path_str, version, "turso history db opened (local)");

        Ok(Self { conn })
    }

    pub async fn record_snapshot(&self, aircraft: &[Aircraft], ts: f64) -> Result<()> {
        let payload = serde_json::to_string(aircraft)?;
        self.conn
            .execute(
                "INSERT INTO snapshots (ts, payload) VALUES (?1, ?2)",
                turso::params![ts, payload],
            )
            .await?;

        for ac in aircraft {
            if let (Some(lat), Some(lon)) = (ac.latitude, ac.longitude) {
                self.conn
                    .execute(
                        "INSERT INTO traces (icao24, ts, lat, lon, alt) VALUES (?1, ?2, ?3, ?4, ?5)",
                        turso::params![
                            ac.icao24.to_hex(),
                            ts,
                            lat,
                            lon,
                            ac.altitude.map(i64::from)
                        ],
                    )
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn prune(&self, older_than_ts: f64) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM snapshots WHERE ts < ?1",
                turso::params![older_than_ts],
            )
            .await?;
        self.conn
            .execute(
                "DELETE FROM traces WHERE ts < ?1",
                turso::params![older_than_ts],
            )
            .await?;
        Ok(())
    }

    pub async fn trace(&self, icao24: &str, since: f64, until: f64) -> Result<Vec<TrackPoint>> {
        let mut rows = self
            .conn
            .query(
                "SELECT ts, lat, lon, alt FROM traces WHERE icao24 = ?1 AND ts >= ?2 AND ts <= ?3 ORDER BY ts",
                turso::params![icao24.to_lowercase(), since, until],
            )
            .await?;

        let mut out = Vec::new();
        while let Some(row) = rows.next().await? {
            out.push(track_point_from_row(&row)?);
        }
        Ok(out)
    }

    /// Persistent tracks for all aircraft in a time window (pTracks equivalent).
    pub async fn ptracks(
        &self,
        since: f64,
        until: f64,
        interval_secs: f64,
    ) -> Result<Vec<TrackPoint>> {
        let interval = interval_secs.max(1.0);
        let mut rows = self
            .conn
            .query(
                r#"
                SELECT ts, lat, lon, alt FROM traces
                WHERE ts >= ?1 AND ts <= ?2
                ORDER BY ts
                LIMIT 200000
                "#,
                turso::params![since, until],
            )
            .await?;

        let mut out = Vec::new();
        let mut last_bucket: Option<i64> = None;
        while let Some(row) = rows.next().await? {
            let pt = track_point_from_row(&row)?;
            let bucket = (pt.ts / interval) as i64;
            if last_bucket != Some(bucket) {
                last_bucket = Some(bucket);
                out.push(pt);
            } else {
                // keep denser sampling within bucket by still adding every Nth —
                // for parity we only keep first per bucket
            }
        }
        Ok(out)
    }

    pub async fn heatmap(&self, since: f64, until: f64, max: i64) -> Result<Vec<TrackPoint>> {
        let mut rows = self
            .conn
            .query(
                "SELECT ts, lat, lon, alt FROM traces WHERE ts >= ?1 AND ts <= ?2 ORDER BY ts DESC LIMIT ?3",
                turso::params![since, until, max],
            )
            .await?;

        let mut out = Vec::new();
        while let Some(row) = rows.next().await? {
            out.push(track_point_from_row(&row)?);
        }
        Ok(out)
    }

    pub async fn replay_snapshots(&self, since: f64, until: f64) -> Result<Vec<HistorySnapshot>> {
        let mut rows = self
            .conn
            .query(
                "SELECT ts, payload FROM snapshots WHERE ts >= ?1 AND ts <= ?2 ORDER BY ts",
                turso::params![since, until],
            )
            .await?;

        let mut out = Vec::new();
        while let Some(row) = rows.next().await? {
            let ts = value_f64(&row.get_value(0)?)?;
            let payload = value_text(&row.get_value(1)?)?;
            if let Ok(aircraft) = serde_json::from_str(&payload) {
                out.push(HistorySnapshot { ts, aircraft });
            }
        }
        Ok(out)
    }

    pub async fn upsert_aircraft_db(&self, icao24: &str, entry: &AircraftDbEntry) -> Result<()> {
        self.conn
            .execute(
                r#"
                INSERT INTO aircraft_db (icao24, registration, typecode, operator, db_flags, type_long, year)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ON CONFLICT(icao24) DO UPDATE SET
                    registration = excluded.registration,
                    typecode = excluded.typecode,
                    operator = excluded.operator,
                    db_flags = excluded.db_flags,
                    type_long = excluded.type_long,
                    year = excluded.year
                "#,
                turso::params![
                    icao24.to_lowercase(),
                    entry.registration.as_deref(),
                    entry.typecode.as_deref(),
                    entry.operator.as_deref(),
                    i64::from(entry.db_flags),
                    entry.type_long.as_deref(),
                    entry.year.as_deref()
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn load_aircraft_db(&self) -> Result<HashMap<Icao24, AircraftDbEntry>> {
        let mut rows = self
            .conn
            .query(
                "SELECT icao24, registration, typecode, operator, db_flags, type_long, year FROM aircraft_db",
                (),
            )
            .await?;

        let mut map = HashMap::new();
        while let Some(row) = rows.next().await? {
            let icao_raw = value_text(&row.get_value(0)?)?;
            let Some(icao) = Icao24::parse(&icao_raw) else {
                continue;
            };
            let db_flags = match row.get_value(4)? {
                turso::Value::Integer(i) if let Ok(u) = u8::try_from(i) => u,
                turso::Value::Integer(i) => i.clamp(0, 255) as u8,
                turso::Value::Real(f) => f.clamp(0.0, 255.0) as u8,
                _ => 0,
            };
            map.insert(
                icao,
                AircraftDbEntry {
                    registration: value_opt_text(&row.get_value(1)?)?,
                    typecode: value_opt_text(&row.get_value(2)?)?,
                    operator: value_opt_text(&row.get_value(3)?)?,
                    db_flags,
                    type_long: value_opt_text(&row.get_value(5)?)?,
                    year: value_opt_text(&row.get_value(6)?)?,
                },
            );
        }
        Ok(map)
    }
}

fn track_point_from_row(row: &turso::Row) -> Result<TrackPoint> {
    Ok(TrackPoint {
        ts: value_f64(&row.get_value(0)?)?,
        lat: value_f64(&row.get_value(1)?)?,
        lon: value_f64(&row.get_value(2)?)?,
        alt: value_opt_i32(&row.get_value(3)?)?,
    })
}

fn value_f64(v: &Value) -> Result<f64> {
    match v {
        Value::Real(f) => Ok(*f),
        Value::Integer(i) => Ok(*i as f64),
        other => Err(anyhow!("expected real, got {other:?}")),
    }
}

fn value_text(v: &Value) -> Result<String> {
    match v {
        Value::Text(s) => Ok(s.clone()),
        other => Err(anyhow!("expected text, got {other:?}")),
    }
}

fn value_opt_text(v: &Value) -> Result<Option<String>> {
    match v {
        Value::Null => Ok(None),
        Value::Text(s) => Ok(Some(s.clone())),
        other => Err(anyhow!("expected text|null, got {other:?}")),
    }
}

fn value_opt_i32(v: &Value) -> Result<Option<i32>> {
    match v {
        Value::Null => Ok(None),
        Value::Integer(i) => Ok(Some(*i as i32)),
        Value::Real(f) => Ok(Some(*f as i32)),
        other => Err(anyhow!("expected int|null, got {other:?}")),
    }
}
