//! Schema migrations like opencode-mem: ordered `PRAGMA user_version` steps.
//! Append new versions only — never edit applied ones.

use anyhow::{anyhow, Result};
use tracing::info;
use turso::Connection;

/// One schema version: description + SQL statements (applied in one write batch).
#[derive(Debug, Clone)]
pub struct SchemaMigration {
    pub version: i32,
    pub description: &'static str,
    pub statements: &'static [&'static str],
}

pub const HISTORY_MIGRATIONS: &[SchemaMigration] = &[
    SchemaMigration {
        version: 1,
        description: "Baseline snapshots, traces, aircraft_db",
        statements: &[
            r#"
        CREATE TABLE IF NOT EXISTS snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ts REAL NOT NULL,
            payload TEXT NOT NULL
        )
        "#,
            "CREATE INDEX IF NOT EXISTS idx_snapshots_ts ON snapshots(ts)",
            r#"
        CREATE TABLE IF NOT EXISTS traces (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            icao24 TEXT NOT NULL,
            ts REAL NOT NULL,
            lat REAL NOT NULL,
            lon REAL NOT NULL,
            alt INTEGER
        )
        "#,
            "CREATE INDEX IF NOT EXISTS idx_traces_icao_ts ON traces(icao24, ts)",
            "CREATE INDEX IF NOT EXISTS idx_traces_ts ON traces(ts)",
            r#"
        CREATE TABLE IF NOT EXISTS aircraft_db (
            icao24 TEXT PRIMARY KEY,
            registration TEXT,
            typecode TEXT,
            operator TEXT
        )
        "#,
        ],
    },
    SchemaMigration {
        version: 2,
        description: "aircraft_db enrichment: db_flags, type_long, year",
        statements: &[
            "ALTER TABLE aircraft_db ADD COLUMN db_flags INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE aircraft_db ADD COLUMN type_long TEXT",
            "ALTER TABLE aircraft_db ADD COLUMN year TEXT",
        ],
    },
];

async fn get_user_version(conn: &Connection) -> Result<i32> {
    let mut rows = conn.query("PRAGMA user_version", ()).await?;
    let Some(row) = rows.next().await? else {
        return Ok(0);
    };
    match row.get_value(0)? {
        turso::Value::Integer(i) => Ok(i as i32),
        turso::Value::Real(f) => Ok(f as i32),
        _ => Ok(0),
    }
}

async fn set_user_version(conn: &Connection, version: i32) -> Result<()> {
    // PRAGMA user_version cannot use bound params; version is our controlled i32.
    conn.execute(format!("PRAGMA user_version = {version}"), ())
        .await?;
    Ok(())
}

/// Apply pending migrations idempotently. Returns the resulting `user_version`.
pub async fn apply_schema_migrations(
    conn: &Connection,
    migrations: &[SchemaMigration],
) -> Result<i32> {
    let mut sorted: Vec<&SchemaMigration> = migrations.iter().collect();
    sorted.sort_by_key(|m| m.version);

    let current = get_user_version(conn).await?;
    let pending: Vec<&&SchemaMigration> = sorted.iter().filter(|m| m.version > current).collect();

    if pending.is_empty() {
        return Ok(current);
    }

    let target = pending.last().map_or(current, |m| m.version);

    for migration in pending {
        for sql in migration.statements {
            conn.execute(*sql, ()).await.map_err(|e| {
                anyhow!(
                    "migration v{} ({}) failed: {e}",
                    migration.version,
                    migration.description
                )
            })?;
        }
        set_user_version(conn, migration.version).await?;
        info!(
            version = migration.version,
            description = migration.description,
            "applied schema migration"
        );
    }

    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use turso::Builder;

    #[tokio::test]
    async fn migrations_apply_idempotently() -> Result<()> {
        let db = Builder::new_local(":memory:").build().await?;
        let conn = db.connect()?;

        let v1 = apply_schema_migrations(&conn, HISTORY_MIGRATIONS).await?;
        assert_eq!(v1, 2);

        let v2 = apply_schema_migrations(&conn, HISTORY_MIGRATIONS).await?;
        assert_eq!(v2, 2);
        assert_eq!(get_user_version(&conn).await?, 2);
        Ok(())
    }
}
