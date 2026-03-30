use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChecksumKind {
    Checksum,
    Parity,
    Xor,
    ModbusCrc16,
}

impl ChecksumKind {
    #[allow(dead_code)]
    pub const ALL: [ChecksumKind; 4] = [
        ChecksumKind::Checksum,
        ChecksumKind::Parity,
        ChecksumKind::Xor,
        ChecksumKind::ModbusCrc16,
    ];

    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            Self::Checksum => "校验和",
            Self::Parity => "奇偶校验",
            Self::Xor => "异或校验",
            Self::ModbusCrc16 => "ModbusCRC16",
        }
    }

    pub fn apply(self, data: &[u8]) -> Vec<u8> {
        match self {
            Self::Checksum => vec![data.iter().fold(0u8, |acc, value| acc.wrapping_add(*value))],
            Self::Parity => {
                vec![(data.iter().fold(0u8, |acc, value| acc.wrapping_add(*value)) % 2)]
            }
            Self::Xor => vec![data.iter().fold(0u8, |acc, value| acc ^ value)],
            Self::ModbusCrc16 => {
                let mut crc = 0xFFFFu16;
                for byte in data {
                    crc ^= *byte as u16;
                    for _ in 0..8 {
                        if crc & 0x0001 != 0 {
                            crc >>= 1;
                            crc ^= 0xA001;
                        } else {
                            crc >>= 1;
                        }
                    }
                }
                vec![(crc & 0xFF) as u8, (crc >> 8) as u8]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ChecksumKind;

    #[test]
    fn modbus_crc_matches_known_value() {
        let bytes = [0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];
        assert_eq!(ChecksumKind::ModbusCrc16.apply(&bytes), vec![0xC5, 0xCD]);
    }
}
