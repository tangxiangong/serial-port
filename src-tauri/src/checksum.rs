/// Sum all bytes modulo 256.
pub fn checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
}

/// Sum all bytes and return 0 or 1 based on parity (even/odd).
pub fn parity_check(data: &[u8]) -> u8 {
    (data.iter().fold(0u32, |acc, &b| acc + b as u32) % 2) as u8
}

/// XOR all bytes together.
pub fn xor_check(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc ^ b)
}

/// Modbus CRC-16 (polynomial 0xA001, initial value 0xFFFF).
/// Returns [low_byte, high_byte].
pub fn modbus_crc16(data: &[u8]) -> [u8; 2] {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc >>= 1;
                crc ^= 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    [(crc & 0xFF) as u8, (crc >> 8) as u8]
}

use serde::{Deserialize, Serialize};

/// Supported checksum algorithms.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckAlgorithm {
    /// No checksum.
    None,
    /// Sum all bytes mod 256.
    Checksum,
    /// Even/odd parity bit.
    Parity,
    /// XOR all bytes.
    Xor,
    /// Modbus CRC-16 (two bytes, little-endian).
    ModbusCrc16,
}

/// Apply the checksum algorithm to `data` and return the checksum bytes.
/// Returns `None` for `CheckAlgorithm::None`.
pub fn apply_check(data: &[u8], algorithm: &CheckAlgorithm) -> Option<Vec<u8>> {
    match algorithm {
        CheckAlgorithm::None => None,
        CheckAlgorithm::Checksum => Some(vec![checksum(data)]),
        CheckAlgorithm::Parity => Some(vec![parity_check(data)]),
        CheckAlgorithm::Xor => Some(vec![xor_check(data)]),
        CheckAlgorithm::ModbusCrc16 => Some(modbus_crc16(data).to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_basic() {
        // 0x01 + 0x02 + 0x03 = 6
        assert_eq!(checksum(&[0x01, 0x02, 0x03]), 6);
    }

    #[test]
    fn test_checksum_overflow() {
        // 0xFF + 0x01 = 0x100, wraps to 0x00
        assert_eq!(checksum(&[0xFF, 0x01]), 0x00);
    }

    #[test]
    fn test_checksum_empty() {
        assert_eq!(checksum(&[]), 0);
    }

    #[test]
    fn test_parity_even() {
        // 1 + 2 = 3 (odd) -> 1
        assert_eq!(parity_check(&[1, 2]), 1);
    }

    #[test]
    fn test_parity_odd_sum() {
        // 1 + 3 = 4 (even) -> 0
        assert_eq!(parity_check(&[1, 3]), 0);
    }

    #[test]
    fn test_parity_empty() {
        assert_eq!(parity_check(&[]), 0);
    }

    #[test]
    fn test_xor_basic() {
        assert_eq!(xor_check(&[0xAA, 0xBB, 0xAA]), 0xBB);
    }

    #[test]
    fn test_xor_self_cancels() {
        assert_eq!(xor_check(&[0xFF, 0xFF]), 0x00);
    }

    #[test]
    fn test_xor_empty() {
        assert_eq!(xor_check(&[]), 0);
    }

    #[test]
    fn test_modbus_crc16_known_value() {
        // Well-known Modbus CRC-16 test vector:
        // "01 03 00 00 00 02" -> CRC = 0xC40B -> [0x0B, 0xC4]
        let data = [0x01u8, 0x03, 0x00, 0x00, 0x00, 0x02];
        // CRC value = 0x0BC4; low byte first -> [0xC4, 0x0B]
        assert_eq!(modbus_crc16(&data), [0xC4, 0x0B]);
    }

    #[test]
    fn test_modbus_crc16_empty() {
        // CRC of empty data with initial value 0xFFFF -> [0xFF, 0xFF]
        assert_eq!(modbus_crc16(&[]), [0xFF, 0xFF]);
    }

    #[test]
    fn test_apply_check_checksum() {
        let result = apply_check(&[1, 2, 3], &CheckAlgorithm::Checksum).unwrap();
        assert_eq!(result, vec![6]);
    }

    #[test]
    fn test_apply_check_parity() {
        let result = apply_check(&[1, 2], &CheckAlgorithm::Parity).unwrap();
        assert_eq!(result, vec![1]);
    }

    #[test]
    fn test_apply_check_xor() {
        let result = apply_check(&[0xAA, 0xBB, 0xAA], &CheckAlgorithm::Xor).unwrap();
        assert_eq!(result, vec![0xBB]);
    }

    #[test]
    fn test_apply_check_modbus_crc16() {
        let data = [0x01u8, 0x03, 0x00, 0x00, 0x00, 0x02];
        let result = apply_check(&data, &CheckAlgorithm::ModbusCrc16).unwrap();
        assert_eq!(result, vec![0xC4, 0x0B]);
    }

    #[test]
    fn test_apply_check_none() {
        assert!(apply_check(&[1, 2, 3], &CheckAlgorithm::None).is_none());
    }
}
