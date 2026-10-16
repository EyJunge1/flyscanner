use thiserror::Error;

const ESC: u8 = 0x1a;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeastMessageType {
    ModeAc,
    ModeSShort,
    ModeSLong,
    Status,
}

impl BeastMessageType {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x31 => Some(Self::ModeAc),
            0x32 => Some(Self::ModeSShort),
            0x33 => Some(Self::ModeSLong),
            0x34 => Some(Self::Status),
            _ => None,
        }
    }

    pub fn as_byte(self) -> u8 {
        match self {
            Self::ModeAc => 0x31,
            Self::ModeSShort => 0x32,
            Self::ModeSLong => 0x33,
            Self::Status => 0x34,
        }
    }

    /// Total framed length including leading ESC + type, after unescaping.
    pub fn framed_len(self) -> usize {
        match self {
            Self::ModeAc => 11,
            Self::ModeSShort => 16,
            Self::ModeSLong => 23,
            Self::Status => 23,
        }
    }

    #[expect(dead_code)]
    pub fn payload_len(self) -> usize {
        match self {
            Self::ModeAc => 2,
            Self::ModeSShort => 7,
            Self::ModeSLong => 14,
            Self::Status => 14,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BeastFrame {
    pub msg_type: BeastMessageType,
    pub mlat_timestamp: u64,
    pub signal_level: u8,
    pub payload: Vec<u8>,
}

impl BeastFrame {
    pub fn mode_s_payload(&self) -> Option<&[u8]> {
        match self.msg_type {
            BeastMessageType::ModeSShort | BeastMessageType::ModeSLong => Some(&self.payload),
            _ => None,
        }
    }

    /// Encode a Mode-S frame as Beast binary (with escapes).
    pub fn encode_mode_s(payload: &[u8], signal_level: u8) -> Vec<u8> {
        let msg_type = if payload.len() <= 7 {
            BeastMessageType::ModeSShort
        } else {
            BeastMessageType::ModeSLong
        };
        let mut out = Vec::with_capacity(32);
        out.push(ESC);
        out.push(msg_type.as_byte());
        // 6-byte MLAT timestamp = 0
        for _ in 0..6 {
            push_escaped(&mut out, 0);
        }
        push_escaped(&mut out, signal_level);
        for &b in payload {
            push_escaped(&mut out, b);
        }
        out
    }
}

fn push_escaped(out: &mut Vec<u8>, b: u8) {
    out.push(b);
    if b == ESC {
        out.push(ESC);
    }
}

#[derive(Debug, Error)]
#[expect(dead_code)]
pub enum FrameError {
    #[error("incomplete beast frame")]
    Incomplete,
    #[error("invalid beast message type: {0:#x}")]
    InvalidType(u8),
}

/// Incremental Beast frame parser with 0x1a unescaping.
#[derive(Debug, Default)]
pub struct FrameParser {
    buf: Vec<u8>,
}

impl FrameParser {
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(256),
        }
    }

    pub fn push(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    pub fn next_frame(&mut self) -> Option<BeastFrame> {
        loop {
            if self.buf.len() < 2 {
                return None;
            }

            let Some(esc_idx) = self.buf.iter().position(|&b| b == ESC) else {
                self.buf.clear();
                return None;
            };
            if esc_idx > 0 {
                self.buf.drain(..esc_idx);
            }

            if self.buf.len() < 2 {
                return None;
            }

            let type_byte = *self.buf.get(1)?;
            let Some(msg_type) = BeastMessageType::from_byte(type_byte) else {
                self.buf.drain(..1);
                continue;
            };

            let need = msg_type.framed_len();
            // Unescape in a working copy up to need bytes of *logical* length
            let (frame_bytes, consumed) = unescape_take(&self.buf, need)?;

            self.buf.drain(..consumed);

            if matches!(msg_type, BeastMessageType::Status) {
                continue;
            }

            // frame_bytes: [ESC, type, mlat[6], signal, payload...]
            if frame_bytes.len() < need {
                continue;
            }

            let mlat = frame_bytes.get(2..8)?;
            let mut mlat_timestamp: u64 = 0;
            for b in mlat {
                mlat_timestamp = (mlat_timestamp << 8) | u64::from(*b);
            }
            let signal_level = *frame_bytes.get(8)?;
            let payload = frame_bytes.get(9..need)?.to_vec();

            return Some(BeastFrame {
                msg_type,
                mlat_timestamp,
                signal_level,
                payload,
            });
        }
    }

    pub fn drain_frames(&mut self) -> Vec<BeastFrame> {
        let mut out = Vec::new();
        while let Some(f) = self.next_frame() {
            out.push(f);
        }
        out
    }
}

/// Take `need` logical bytes from `buf`, unescaping `0x1a 0x1a` → `0x1a`.
/// Returns (unescaped bytes, bytes consumed from buf).
fn unescape_take(buf: &[u8], need: usize) -> Option<(Vec<u8>, usize)> {
    let mut out = Vec::with_capacity(need);
    let mut i = 0;
    while out.len() < need {
        if i >= buf.len() {
            return None;
        }
        let b = *buf.get(i)?;
        i += 1;
        out.push(b);
        if b == ESC && out.len() > 1 {
            // After the leading ESC, doubled ESC means one literal ESC.
            // Leading ESC at index 0 of the frame is the start marker and is
            // not doubled with the type byte. Within payload, 0x1a 0x1a → one.
            if buf.get(i).copied() == Some(ESC) {
                i += 1; // consume the escape duplicate, keep single ESC in out
            }
        }
    }
    Some((out, i))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches;

    #[allow(clippy::expect_used)]
    fn next_frame_or_fail(parser: &mut FrameParser) -> BeastFrame {
        parser
            .next_frame()
            .expect("expected beast frame in test input")
    }

    #[test]
    fn parse_long_mode_s() {
        // ESC + type3 + 6 mlat + signal + 14 payload
        let mut raw = vec![0x1a, 0x33];
        raw.extend_from_slice(&[0; 6]);
        raw.push(0xaa);
        raw.extend_from_slice(&[0x8d; 14]);

        let mut parser = FrameParser::new();
        parser.push(&raw);
        let frame = next_frame_or_fail(&mut parser);
        assert_matches!(frame.msg_type, BeastMessageType::ModeSLong);
        assert_eq!(frame.signal_level, 0xaa);
        assert_eq!(frame.payload.len(), 14);
    }

    #[test]
    fn roundtrip_escape() {
        let payload = vec![0x8d, 0x1a, 0x01, 0x02, 0x03, 0x04, 0x05];
        let encoded = BeastFrame::encode_mode_s(&payload, 0x10);
        let mut parser = FrameParser::new();
        parser.push(&encoded);
        let frame = next_frame_or_fail(&mut parser);
        assert_eq!(frame.payload, payload);
    }
}
