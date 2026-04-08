use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    checksum::{CheckAlgorithm, apply_check},
    codec::{ascii_string_to_bytes, dec_string_to_bytes, hex_string_to_bytes},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SendFormat {
    Hex,
    Ascii,
    Dec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LineEnding {
    None,
    /// Line feed `\n`
    LF,
    /// Carriage return `\r`
    CR,
    #[allow(clippy::upper_case_acronyms)]
    /// Carriage return + line feed `\r\n`
    CRLF,
    /// Arbitrary byte sequence encoded as a string.
    Custom(String),
}

impl LineEnding {
    /// Return the line-ending bytes. `LineEnding::None` returns an empty vec.
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            LineEnding::None => vec![],
            LineEnding::LF => b"\n".to_vec(),
            LineEnding::CR => b"\r".to_vec(),
            LineEnding::CRLF => b"\r\n".to_vec(),
            LineEnding::Custom(s) => s.as_bytes().to_vec(),
        }
    }
}

/// Build the final send buffer from user input.
///
/// Steps:
/// 1. Parse `input` according to `format`.
///    - `Hex`  — strip spaces then decode as hex pairs.
///    - `Ascii`— append the line ending to the raw text *before* encoding.
///    - `Dec`  — parse space-separated decimal values (0-255).
/// 2. For `Hex` and `Dec`, the line ending is **not** appended (matches JS
///    send-store behaviour where line ending only applies to ASCII mode).
/// 3. If `check_algorithm` is non-empty, compute the checksum over the
///    assembled buffer and append the result bytes.
pub fn build_send_buffer(
    input: &str,
    format: &SendFormat,
    line_ending: &LineEnding,
    check_algorithm: &CheckAlgorithm,
) -> Result<Vec<u8>> {
    let mut buffer: Vec<u8> = match format {
        SendFormat::Hex => {
            let stripped = input.replace(' ', "");
            hex_string_to_bytes(&stripped)?
        }
        SendFormat::Ascii => {
            let le_str = String::from_utf8(line_ending.as_bytes()).unwrap_or_default();
            let combined = format!("{}{}", input, le_str);
            ascii_string_to_bytes(&combined)
        }
        SendFormat::Dec => dec_string_to_bytes(input)?,
    };

    if let Some(checksum_bytes) = apply_check(&buffer, check_algorithm) {
        buffer.extend_from_slice(&checksum_bytes);
    }

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONE: &CheckAlgorithm = &CheckAlgorithm::None;

    // ── LineEnding::as_bytes ─────────────────────────────────────────────────

    #[test]
    fn test_line_ending_none_is_empty() {
        assert!(LineEnding::None.as_bytes().is_empty());
    }

    #[test]
    fn test_line_ending_lf() {
        assert_eq!(LineEnding::LF.as_bytes(), b"\n");
    }

    #[test]
    fn test_line_ending_cr() {
        assert_eq!(LineEnding::CR.as_bytes(), b"\r");
    }

    #[test]
    fn test_line_ending_crlf() {
        assert_eq!(LineEnding::CRLF.as_bytes(), b"\r\n");
    }

    #[test]
    fn test_line_ending_custom() {
        assert_eq!(LineEnding::Custom("END".to_string()).as_bytes(), b"END");
    }

    // ── build_send_buffer — Hex format ───────────────────────────────────────

    #[test]
    fn test_hex_basic() {
        let buf = build_send_buffer("AABB CC", &SendFormat::Hex, &LineEnding::None, NONE).unwrap();
        assert_eq!(buf, vec![0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_hex_line_ending_not_appended() {
        let buf_none =
            build_send_buffer("AABB", &SendFormat::Hex, &LineEnding::None, NONE).unwrap();
        let buf_lf = build_send_buffer("AABB", &SendFormat::Hex, &LineEnding::LF, NONE).unwrap();
        assert_eq!(buf_none, buf_lf);
    }

    #[test]
    fn test_hex_invalid_input_returns_error() {
        assert!(build_send_buffer("GG", &SendFormat::Hex, &LineEnding::None, NONE).is_err());
    }

    #[test]
    fn test_hex_odd_length_returns_error() {
        assert!(build_send_buffer("AAB", &SendFormat::Hex, &LineEnding::None, NONE).is_err());
    }

    // ── build_send_buffer — ASCII format ─────────────────────────────────────

    #[test]
    fn test_ascii_basic() {
        let buf = build_send_buffer("hello", &SendFormat::Ascii, &LineEnding::None, NONE).unwrap();
        assert_eq!(buf, b"hello");
    }

    #[test]
    fn test_ascii_with_lf_line_ending() {
        let buf = build_send_buffer("AT", &SendFormat::Ascii, &LineEnding::LF, NONE).unwrap();
        assert_eq!(buf, b"AT\n");
    }

    #[test]
    fn test_ascii_with_crlf_line_ending() {
        let buf = build_send_buffer("AT", &SendFormat::Ascii, &LineEnding::CRLF, NONE).unwrap();
        assert_eq!(buf, b"AT\r\n");
    }

    #[test]
    fn test_ascii_with_custom_line_ending() {
        let buf = build_send_buffer(
            "cmd",
            &SendFormat::Ascii,
            &LineEnding::Custom(";".to_string()),
            NONE,
        )
        .unwrap();
        assert_eq!(buf, b"cmd;");
    }

    // ── build_send_buffer — Dec format ───────────────────────────────────────

    #[test]
    fn test_dec_basic() {
        let buf =
            build_send_buffer("255 0 128", &SendFormat::Dec, &LineEnding::None, NONE).unwrap();
        assert_eq!(buf, vec![0xFF, 0x00, 0x80]);
    }

    #[test]
    fn test_dec_line_ending_not_appended() {
        let buf_none =
            build_send_buffer("1 2 3", &SendFormat::Dec, &LineEnding::None, NONE).unwrap();
        let buf_crlf =
            build_send_buffer("1 2 3", &SendFormat::Dec, &LineEnding::CRLF, NONE).unwrap();
        assert_eq!(buf_none, buf_crlf);
    }

    #[test]
    fn test_dec_out_of_range_returns_error() {
        assert!(build_send_buffer("256", &SendFormat::Dec, &LineEnding::None, NONE).is_err());
    }

    // ── build_send_buffer — checksum ─────────────────────────────────────────

    #[test]
    fn test_checksum_appended() {
        let buf = build_send_buffer(
            "01 02 03",
            &SendFormat::Hex,
            &LineEnding::None,
            &CheckAlgorithm::Checksum,
        )
        .unwrap();
        assert_eq!(buf, vec![0x01, 0x02, 0x03, 0x06]);
    }

    #[test]
    fn test_xor_appended() {
        let buf = build_send_buffer(
            "AA BB AA",
            &SendFormat::Hex,
            &LineEnding::None,
            &CheckAlgorithm::Xor,
        )
        .unwrap();
        assert_eq!(buf, vec![0xAA, 0xBB, 0xAA, 0xBB]);
    }

    #[test]
    fn test_modbus_crc16_appended() {
        let buf = build_send_buffer(
            "01 03 00 00 00 02",
            &SendFormat::Hex,
            &LineEnding::None,
            &CheckAlgorithm::ModbusCrc16,
        )
        .unwrap();
        assert_eq!(buf, vec![0x01, 0x03, 0x00, 0x00, 0x00, 0x02, 0xC4, 0x0B]);
    }

    #[test]
    fn test_none_algorithm_no_checksum() {
        let buf = build_send_buffer("AA BB", &SendFormat::Hex, &LineEnding::None, NONE).unwrap();
        assert_eq!(buf, vec![0xAA, 0xBB]);
    }

    #[test]
    fn test_ascii_with_checksum() {
        let buf = build_send_buffer(
            "AB",
            &SendFormat::Ascii,
            &LineEnding::None,
            &CheckAlgorithm::Checksum,
        )
        .unwrap();
        assert_eq!(buf, vec![0x41, 0x42, 0x83]);
    }
}
