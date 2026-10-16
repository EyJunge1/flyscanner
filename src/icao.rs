//! ICAO24 Mode-S address as a fixed 3-byte newtype.
//!
//! Wire/JSON representation stays a 6-char lowercase hex string so the
//! frontend and SQLite schema stay unchanged.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// 24-bit Mode-S address (`XXYYYY` hex on the wire).
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Icao24(pub [u8; 3]);

impl Icao24 {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 3]) -> Self {
        Self(bytes)
    }

    /// Parse 6-char hex (case-insensitive). Returns `None` on bad input.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        let bytes = s.trim().as_bytes().as_array::<6>()?;
        let b0 = u8::from_str_radix(std::str::from_utf8(bytes.get(0..2)?).ok()?, 16).ok()?;
        let b1 = u8::from_str_radix(std::str::from_utf8(bytes.get(2..4)?).ok()?, 16).ok()?;
        let b2 = u8::from_str_radix(std::str::from_utf8(bytes.get(4..6)?).ok()?, 16).ok()?;
        Some(Self([b0, b1, b2]))
    }

    #[must_use]
    pub fn to_hex(self) -> String {
        format!("{:02x}{:02x}{:02x}", self.0[0], self.0[1], self.0[2])
    }
}

impl fmt::Display for Icao24 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}{:02x}{:02x}",
            self.0[0], self.0[1], self.0[2]
        )
    }
}

impl fmt::Debug for Icao24 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Icao24({self})")
    }
}

impl Serialize for Icao24 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Icao24 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).ok_or_else(|| serde::de::Error::custom(format!("invalid icao24: {s}")))
    }
}

impl FromStr for Icao24 {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or("invalid icao24")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches;

    #[test]
    fn roundtrip_hex() {
        let icao = Icao24::from_bytes([0x3c, 0x66, 0xe6]);
        assert_matches!(Icao24::parse("3C66E6"), Some(parsed) if parsed == icao);
        assert_eq!(icao.to_hex(), "3c66e6");
        assert_eq!(icao.0, [0x3c, 0x66, 0xe6]);
    }

    #[test]
    fn rejects_bad() {
        assert_matches!(Icao24::parse("gg0000"), None);
        assert_matches!(Icao24::parse("abc"), None);
    }
}
