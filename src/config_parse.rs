//! Parse Ultrafeeder-like `FLYSCANNER_CONFIG` strings.
//!
//! Supported constructs (semicolon-separated):
//! - `adsb,<host>,<port>,beast_out` — full-rate Beast push
//! - `adsb,<host>,<port>,beast_reduce_out` — rate-limited Beast
//! - `adsb,<host>,<port>,beast_reduce_plus_out[,uuid=<UUID>]` — reduce + UUID handshake
//! - `adsb,<host>,<port>,beast_in` — ingest Beast from remote
//!
//! `mlat,...` / `mlathub,...` are parsed and logged; use feeder sidecars for MLAT clients.

use crate::beast::FeedMode;
use crate::feeder::SourceConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedTarget {
    pub host: String,
    pub port: u16,
    pub mode: FeedMode,
    pub uuid: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct ParsedConfig {
    pub ingest: Vec<SourceConfig>,
    pub feed_out: Vec<FeedTarget>,
    pub skipped: Vec<String>,
}

/// Parse `FLYSCANNER_CONFIG` / Ultrafeeder-style config blob.
pub fn parse_flyscanner_config(raw: &str) -> ParsedConfig {
    let mut out = ParsedConfig::default();
    for part in raw.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let fields: Vec<&str> = part.split(',').map(str::trim).collect();
        let Some(kind) = fields.first().map(|s| s.to_ascii_lowercase()) else {
            continue;
        };
        match kind.as_str() {
            "adsb" => {
                let (Some(host), Some(port_s), Some(proto_s)) =
                    (fields.get(1), fields.get(2), fields.get(3))
                else {
                    out.skipped.push(part.to_string());
                    continue;
                };
                let host = (*host).to_string();
                let Ok(port) = port_s.parse::<u16>() else {
                    out.skipped.push(part.to_string());
                    continue;
                };
                let proto = proto_s.to_ascii_lowercase();
                if proto.contains("beast") && proto.contains("in") {
                    out.ingest.push(SourceConfig {
                        name: format!("cfg-{host}-{port}"),
                        host,
                        port,
                        is_mlat: false,
                    });
                } else if proto.contains("beast") {
                    let mode = if proto.contains("reduce_plus") || proto.contains("reduce-plus") {
                        FeedMode::BeastReducePlus
                    } else if proto.contains("reduce") {
                        FeedMode::BeastReduce
                    } else {
                        FeedMode::Beast
                    };
                    let uuid = fields.iter().find_map(|f| {
                        let f = f.trim();
                        f.strip_prefix("uuid=")
                            .or_else(|| f.strip_prefix("UUID="))
                            .map(str::to_string)
                    });
                    out.feed_out.push(FeedTarget {
                        host,
                        port,
                        mode,
                        uuid,
                    });
                } else {
                    out.skipped.push(part.to_string());
                }
            }
            "mlat" | "mlathub" => {
                out.skipped.push(part.to_string());
            }
            _ => out.skipped.push(part.to_string()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches;

    #[test]
    fn parses_ultrafeeder_style_adsb() {
        let cfg = parse_flyscanner_config(
            "adsb,feed.adsb.fi,30004,beast_reduce_plus_out,uuid=aabbccdd-eeff-0011-2233-445566778899;adsb,readsb,30005,beast_in;mlat,in.adsb.lol,31090;",
        );
        assert_eq!(cfg.feed_out.len(), 1);
        let feed = cfg.feed_out.first();
        assert_eq!(feed.map(|t| t.host.as_str()), Some("feed.adsb.fi"));
        assert_eq!(feed.map(|t| t.port), Some(30004));
        assert_matches!(feed.map(|t| t.mode), Some(FeedMode::BeastReducePlus));
        assert_matches!(
            feed.and_then(|t| t.uuid.as_deref()),
            Some("aabbccdd-eeff-0011-2233-445566778899")
        );
        assert_eq!(cfg.ingest.len(), 1);
        assert_eq!(cfg.ingest.first().map(|s| s.host.as_str()), Some("readsb"));
        assert_eq!(cfg.skipped.len(), 1);
        assert!(cfg.skipped.first().is_some_and(|s| s.starts_with("mlat")));
    }

    #[test]
    fn parses_beast_reduce_without_plus() {
        let cfg = parse_flyscanner_config("adsb,example.com,30004,beast_reduce_out;");
        assert_matches!(
            cfg.feed_out.first().map(|t| t.mode),
            Some(FeedMode::BeastReduce)
        );
    }
}
