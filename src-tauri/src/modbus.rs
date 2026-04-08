use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModbusFrame {
    pub slave_id: u8,
    pub function_code: u8,
    pub function_name: String,
    pub data: Vec<u8>,
    pub crc_valid: bool,
    pub raw: Vec<u8>,
}

/// Parse Modbus RTU frame from raw bytes.
/// RTU format: [slave_id, function_code, data..., crc_lo, crc_hi]
pub fn parse_rtu_frame(data: &[u8]) -> Result<ModbusFrame> {
    if data.len() < 4 {
        return Err(anyhow!("Frame too short"));
    }
    let slave_id = data[0];
    let function_code = data[1];
    let payload = &data[2..data.len() - 2];
    let crc_received = [data[data.len() - 2], data[data.len() - 1]];
    let crc_calculated = crate::checksum::modbus_crc16(&data[..data.len() - 2]);
    let crc_valid = crc_received == crc_calculated;

    Ok(ModbusFrame {
        slave_id,
        function_code,
        function_name: function_code_name(function_code),
        data: payload.to_vec(),
        crc_valid,
        raw: data.to_vec(),
    })
}

/// Parse Modbus ASCII frame from raw bytes.
/// ASCII format: ':' + hex pairs + LRC + CRLF
/// Input should be the raw bytes including ':' prefix and CRLF suffix.
pub fn parse_ascii_frame(data: &[u8]) -> Result<ModbusFrame> {
    // Validate frame structure
    if data.len() < 9 {
        // Minimum: ':' + 2 hex (slave) + 2 hex (func) + 2 hex (LRC) + CR + LF = 9
        return Err(anyhow!("ASCII frame too short"));
    }
    if data[0] != b':' {
        return Err(anyhow!("ASCII frame must start with ':'"));
    }
    // Check CRLF suffix
    if data[data.len() - 2] != b'\r' || data[data.len() - 1] != b'\n' {
        return Err(anyhow!("ASCII frame must end with CRLF"));
    }

    // Extract hex content between ':' and CRLF
    let hex_content = &data[1..data.len() - 2];

    // Validate that all characters are valid hex
    if !hex_content.iter().all(|&b| b.is_ascii_hexdigit()) {
        return Err(anyhow!("ASCII frame contains non-hex characters"));
    }
    if !hex_content.len().is_multiple_of(2) {
        return Err(anyhow!("ASCII frame hex content has odd length"));
    }
    if hex_content.len() < 6 {
        // At least slave_id(2) + function_code(2) + lrc(2)
        return Err(anyhow!("ASCII frame hex content too short"));
    }

    // Decode hex pairs to bytes
    let hex_str = std::str::from_utf8(hex_content)
        .map_err(|e| anyhow!("Invalid UTF-8 in hex content: {e}"))?;
    let decoded = crate::codec::hex_string_to_bytes(hex_str)?;

    // Last byte is LRC, everything before is the message
    let message = &decoded[..decoded.len() - 1];
    let received_lrc = decoded[decoded.len() - 1];
    let calculated_lrc = lrc(message);
    let crc_valid = received_lrc == calculated_lrc;

    let slave_id = message[0];
    let function_code = message[1];
    let payload = &message[2..];

    Ok(ModbusFrame {
        slave_id,
        function_code,
        function_name: function_code_name(function_code),
        data: payload.to_vec(),
        crc_valid,
        raw: data.to_vec(),
    })
}

fn function_code_name(code: u8) -> String {
    match code {
        0x01 => "Read Coils".into(),
        0x02 => "Read Discrete Inputs".into(),
        0x03 => "Read Holding Registers".into(),
        0x04 => "Read Input Registers".into(),
        0x05 => "Write Single Coil".into(),
        0x06 => "Write Single Register".into(),
        0x0F => "Write Multiple Coils".into(),
        0x10 => "Write Multiple Registers".into(),
        code if code >= 0x80 => format!("Exception (0x{:02X})", code),
        _ => format!("Unknown (0x{:02X})", code),
    }
}

fn lrc(data: &[u8]) -> u8 {
    let sum: u8 = data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    (!sum).wrapping_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rtu_frame_valid() {
        // slave=0x01, func=0x03, data=[0x00, 0x00, 0x00, 0x02]
        // CRC of [0x01, 0x03, 0x00, 0x00, 0x00, 0x02] = [0xC4, 0x0B]
        let frame = [0x01, 0x03, 0x00, 0x00, 0x00, 0x02, 0xC4, 0x0B];
        let result = parse_rtu_frame(&frame).unwrap();
        assert_eq!(result.slave_id, 0x01);
        assert_eq!(result.function_code, 0x03);
        assert_eq!(result.function_name, "Read Holding Registers");
        assert_eq!(result.data, vec![0x00, 0x00, 0x00, 0x02]);
        assert!(result.crc_valid);
    }

    #[test]
    fn test_parse_rtu_frame_invalid_crc() {
        let frame = [0x01, 0x03, 0x00, 0x00, 0x00, 0x02, 0xFF, 0xFF];
        let result = parse_rtu_frame(&frame).unwrap();
        assert!(!result.crc_valid);
    }

    #[test]
    fn test_parse_rtu_frame_too_short() {
        assert!(parse_rtu_frame(&[0x01, 0x03, 0x00]).is_err());
    }

    #[test]
    fn test_parse_rtu_frame_minimum_length() {
        // 4 bytes: slave + func + crc_lo + crc_hi, no data payload
        let data_without_crc = [0x01u8, 0x03];
        let crc = crate::checksum::modbus_crc16(&data_without_crc);
        let frame = [0x01, 0x03, crc[0], crc[1]];
        let result = parse_rtu_frame(&frame).unwrap();
        assert_eq!(result.slave_id, 0x01);
        assert_eq!(result.function_code, 0x03);
        assert!(result.data.is_empty());
        assert!(result.crc_valid);
    }

    #[test]
    fn test_lrc() {
        // LRC of [0x01, 0x03, 0x00, 0x00, 0x00, 0x02]
        // sum = 0x06, !0x06 = 0xF9, +1 = 0xFA
        assert_eq!(lrc(&[0x01, 0x03, 0x00, 0x00, 0x00, 0x02]), 0xFA);
    }

    #[test]
    fn test_parse_ascii_frame_valid() {
        // Build: ":0103000000020xFA\r\n"
        // Message bytes: [0x01, 0x03, 0x00, 0x00, 0x00, 0x02], LRC = 0xFA
        let frame = b":010300000002FA\r\n";
        let result = parse_ascii_frame(frame).unwrap();
        assert_eq!(result.slave_id, 0x01);
        assert_eq!(result.function_code, 0x03);
        assert_eq!(result.function_name, "Read Holding Registers");
        assert_eq!(result.data, vec![0x00, 0x00, 0x00, 0x02]);
        assert!(result.crc_valid);
    }

    #[test]
    fn test_parse_ascii_frame_invalid_lrc() {
        let frame = b":010300000002FF\r\n";
        let result = parse_ascii_frame(frame).unwrap();
        assert!(!result.crc_valid);
    }

    #[test]
    fn test_parse_ascii_frame_no_colon() {
        assert!(parse_ascii_frame(b"010300000002FA\r\n").is_err());
    }

    #[test]
    fn test_parse_ascii_frame_no_crlf() {
        assert!(parse_ascii_frame(b":010300000002FA").is_err());
    }

    #[test]
    fn test_parse_ascii_frame_too_short() {
        assert!(parse_ascii_frame(b":01\r\n").is_err());
    }

    #[test]
    fn test_function_code_names() {
        assert_eq!(function_code_name(0x01), "Read Coils");
        assert_eq!(function_code_name(0x05), "Write Single Coil");
        assert_eq!(function_code_name(0x10), "Write Multiple Registers");
        assert_eq!(function_code_name(0x83), "Exception (0x83)");
        assert_eq!(function_code_name(0x07), "Unknown (0x07)");
    }
}
