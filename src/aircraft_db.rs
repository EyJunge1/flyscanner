//! Aircraft metadata DB — tar1090-db / readsb compatible `aircraft.csv[.gz]`.
//!
//! Format (semicolon-separated, no header):
//! `icao24;registration;typeCode;dbFlags;typeLong;year;ownOp;`

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use flate2::read::MultiGzDecoder;
use tracing::info;

use crate::icao::Icao24;
use crate::state::AircraftDbEntry;

/// Load a tar1090-db CSV (plain or `.gz`). Empty fields become `None`.
pub fn load_from_path(path: impl AsRef<Path>) -> Result<HashMap<Icao24, AircraftDbEntry>> {
    let path = path.as_ref();
    let file = File::open(path).with_context(|| format!("open aircraft db {}", path.display()))?;
    let reader: Box<dyn Read> = if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("gz"))
    {
        Box::new(MultiGzDecoder::new(file))
    } else {
        Box::new(file)
    };

    let map = load_from_reader(BufReader::new(reader))?;
    info!(path = %path.display(), entries = map.len(), "aircraft db loaded");
    Ok(map)
}

/// Parse CSV content from any buffered reader.
pub fn load_from_reader(reader: impl BufRead) -> Result<HashMap<Icao24, AircraftDbEntry>> {
    let mut map = HashMap::new();
    for (line_no, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("read aircraft db line {}", line_no + 1))?;
        if line.is_empty() {
            continue;
        }
        if let Some((icao, entry)) = parse_line(&line) {
            map.insert(icao, entry);
        }
    }
    (!map.is_empty())
        .ok_or_else(|| anyhow!("aircraft db has no entries (wrong format?)"))?;
    Ok(map)
}

fn parse_line(line: &str) -> Option<(Icao24, AircraftDbEntry)> {
    // Keep trailing empties so field indices stay stable.
    let mut parts = line.split(';');
    let icao_raw = parts.next()?.trim();
    (!icao_raw.is_empty()).then_some(())?;
    // Skip bogus / zero addresses (same as readsb).
    let addr = u32::from_str_radix(icao_raw, 16).ok()?;
    (addr != 0).then_some(())?;
    let icao = Icao24::parse(icao_raw)?;

    let registration = nonempty(parts.next());
    let typecode = nonempty(parts.next());
    let db_flags = parse_db_flags(parts.next());
    let type_long = nonempty(parts.next());
    let year = nonempty(parts.next());
    let operator = nonempty(parts.next());

    Some((
        icao,
        AircraftDbEntry {
            registration,
            typecode,
            type_long,
            year,
            operator,
            db_flags,
        },
    ))
}

/// Parse hex or decimal dbFlags (tar1090-db / readsb). Empty → 0.
fn parse_db_flags(raw: Option<&str>) -> u8 {
    let Some(s) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return 0;
    };
    // CSV stores hex without prefix (`00`, `01`, `0a`). Fall back to decimal.
    u8::from_str_radix(s, 16).unwrap_or_else(|_| s.parse().unwrap_or(0))
}

fn nonempty(s: Option<&str>) -> Option<String> {
    s.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parses_tar1090_style_row() {
        let csv = "\
3c66e6;D-AIMA;A320;00;AIRBUS A-320;;Lufthansa;\n\
000000;;;;;\n\
a37c5a;N326SB;C172;00;;;OWNER;\n";
        let map = load_from_reader(Cursor::new(csv.as_bytes())).unwrap_or_default();
        let icao_lh = Icao24::from_bytes([0x3c, 0x66, 0xe6]);
        let icao_n = Icao24::from_bytes([0xa3, 0x7c, 0x5a]);
        assert_eq!(map.len(), 2);
        assert_eq!(
            map.get(&icao_lh).and_then(|e| e.registration.as_deref()),
            Some("D-AIMA")
        );
        assert_eq!(
            map.get(&icao_lh).and_then(|e| e.typecode.as_deref()),
            Some("A320")
        );
        assert_eq!(
            map.get(&icao_lh).and_then(|e| e.operator.as_deref()),
            Some("Lufthansa")
        );
        assert_eq!(
            map.get(&icao_lh).and_then(|e| e.type_long.as_deref()),
            Some("AIRBUS A-320")
        );
        assert_eq!(map.get(&icao_lh).map(|e| e.db_flags), Some(0));
        assert_eq!(
            map.get(&icao_n).and_then(|e| e.registration.as_deref()),
            Some("N326SB")
        );
        assert_eq!(
            map.get(&icao_n).and_then(|e| e.operator.as_deref()),
            Some("OWNER")
        );
    }

    #[test]
    fn parses_db_flags_military() {
        let csv = "3c0001;D-MIL;F16;01;FIGHTER;1990;Luftwaffe;\n";
        let map = load_from_reader(Cursor::new(csv.as_bytes())).unwrap_or_default();
        let icao = Icao24::from_bytes([0x3c, 0x00, 0x01]);
        assert_eq!(map.get(&icao).map(|e| e.db_flags), Some(1));
        assert_eq!(
            map.get(&icao).and_then(|e| e.type_long.as_deref()),
            Some("FIGHTER")
        );
        assert_eq!(map.get(&icao).and_then(|e| e.year.as_deref()), Some("1990"));
    }
}
