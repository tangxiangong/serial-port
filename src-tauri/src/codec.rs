use anyhow::{Result, anyhow};

/// Convert a hex string to bytes. Spaces are stripped before parsing.
/// Both "AABBCC" and "AA BB CC" are accepted.
pub fn hex_string_to_bytes(s: &str) -> Result<Vec<u8>> {
    let stripped: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if stripped.is_empty() {
        return Ok(Vec::new());
    }
    if !stripped.len().is_multiple_of(2) {
        return Err(anyhow!(
            "hex string has odd length after stripping spaces: {}",
            stripped
        ));
    }
    stripped
        .as_bytes()
        .chunks(2)
        .map(|chunk| {
            let pair = std::str::from_utf8(chunk)
                .map_err(|e| anyhow!("invalid UTF-8 in hex string: {e}"))?;
            u8::from_str_radix(pair, 16).map_err(|e| anyhow!("invalid hex byte '{pair}': {e}"))
        })
        .collect()
}

/// Convert bytes to an uppercase hex string with no separators. [0xAA, 0xBB] -> "AABB"
pub fn bytes_to_hex_string(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02X}")).collect()
}

/// Convert bytes to "0xAA, 0xBB" display format.
pub fn bytes_to_hex_display(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("0x{b:02X}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Convert bytes to "AA BB CC" space-separated hex format.
pub fn bytes_to_hex_spaced(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Encode a UTF-8 string to bytes.
pub fn ascii_string_to_bytes(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

/// Decode bytes to a UTF-8 string (lossy — invalid sequences replaced with U+FFFD).
pub fn bytes_to_ascii_string(data: &[u8]) -> String {
    String::from_utf8_lossy(data).into_owned()
}

/// Parse a space-separated decimal string into bytes. Each token must be 0–255.
/// Example: "255 0 128" -> [0xFF, 0x00, 0x80]
pub fn dec_string_to_bytes(s: &str) -> Result<Vec<u8>> {
    s.split_whitespace()
        .map(|token| {
            let n: u16 = token
                .parse()
                .map_err(|e| anyhow!("invalid decimal value '{token}': {e}"))?;
            if n > 255 {
                return Err(anyhow!("decimal value {n} out of range 0-255"));
            }
            Ok(n as u8)
        })
        .collect()
}

/// Convert bytes to a space-separated decimal string. [0xFF, 0x00] -> "255 0"
pub fn bytes_to_dec_string(data: &[u8]) -> String {
    data.iter()
        .map(|b| b.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Return true if the string contains only hex characters (0-9, a-f, A-F).
pub fn is_hex_string(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Reformat a hex input string into canonical "AA BB CC" form.
///
/// Each whitespace-separated token is zero-padded to even length, then any
/// remaining characters are split into 2-char groups and joined with spaces.
///
/// Examples:
///   "A BB  C"  -> "0A BB 0C"
///   "AABBCC"   -> "AA BB CC"
///   "A B C"    -> "0A 0B 0C"
pub fn format_hex_input(input: &str) -> String {
    let mut groups: Vec<String> = Vec::new();
    for token in input.split_whitespace() {
        // Pad odd-length tokens with a leading zero.
        let padded = if token.len() % 2 != 0 {
            format!("0{token}")
        } else {
            token.to_string()
        };
        // Split into 2-char chunks.
        for chunk in padded.as_bytes().chunks(2) {
            if let Ok(s) = std::str::from_utf8(chunk) {
                groups.push(s.to_uppercase());
            }
        }
    }
    groups.join(" ")
}

/// Return true if the input is a valid hex byte string (ignoring spaces).
///
/// Valid inputs: empty string, or a sequence of hex pairs optionally separated
/// by spaces. An odd number of hex digits is accepted for in-progress input.
pub fn validate_hex_input(input: &str) -> bool {
    let stripped: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    stripped.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_string_to_bytes_packed() {
        assert_eq!(
            hex_string_to_bytes("AABBCC").unwrap(),
            vec![0xAA, 0xBB, 0xCC]
        );
    }

    #[test]
    fn test_hex_string_to_bytes_spaced() {
        assert_eq!(
            hex_string_to_bytes("AA BB CC").unwrap(),
            vec![0xAA, 0xBB, 0xCC]
        );
    }

    #[test]
    fn test_hex_string_to_bytes_empty() {
        assert_eq!(hex_string_to_bytes("").unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn test_hex_string_to_bytes_odd_length() {
        assert!(hex_string_to_bytes("ABC").is_err());
    }

    #[test]
    fn test_bytes_to_hex_string() {
        assert_eq!(bytes_to_hex_string(&[0xAA, 0xBB, 0xCC]), "AABBCC");
        assert_eq!(bytes_to_hex_string(&[0x0F]), "0F");
    }

    #[test]
    fn test_bytes_to_hex_display() {
        assert_eq!(bytes_to_hex_display(&[0xAA, 0xBB]), "0xAA, 0xBB");
    }

    #[test]
    fn test_bytes_to_hex_spaced() {
        assert_eq!(bytes_to_hex_spaced(&[0xAA, 0xBB, 0xCC]), "AA BB CC");
    }

    #[test]
    fn test_ascii_roundtrip() {
        let s = "Hello, 世界!";
        let bytes = ascii_string_to_bytes(s);
        assert_eq!(bytes_to_ascii_string(&bytes), s);
    }

    #[test]
    fn test_dec_string_to_bytes() {
        assert_eq!(
            dec_string_to_bytes("255 0 128").unwrap(),
            vec![0xFF, 0x00, 0x80]
        );
    }

    #[test]
    fn test_dec_string_to_bytes_out_of_range() {
        assert!(dec_string_to_bytes("256").is_err());
    }

    #[test]
    fn test_bytes_to_dec_string() {
        assert_eq!(bytes_to_dec_string(&[0xFF, 0x00]), "255 0");
    }

    #[test]
    fn test_dec_roundtrip() {
        let data: Vec<u8> = vec![0, 1, 127, 128, 255];
        let s = bytes_to_dec_string(&data);
        assert_eq!(dec_string_to_bytes(&s).unwrap(), data);
    }

    #[test]
    fn test_is_hex_string() {
        assert!(is_hex_string("0123456789abcdefABCDEF"));
        assert!(!is_hex_string("GG"));
        assert!(!is_hex_string(""));
        assert!(!is_hex_string("AA BB")); // spaces not allowed
    }

    #[test]
    fn test_format_hex_input() {
        assert_eq!(format_hex_input("A BB  C"), "0A BB 0C");
        assert_eq!(format_hex_input("AABBCC"), "AA BB CC");
        assert_eq!(format_hex_input("A B C"), "0A 0B 0C");
        assert_eq!(format_hex_input(""), "");
    }

    #[test]
    fn test_validate_hex_input() {
        assert!(validate_hex_input("AABBCC"));
        assert!(validate_hex_input("AA BB CC")); // spaces stripped
        assert!(validate_hex_input("ABC")); // odd digit count accepted
        assert!(validate_hex_input(""));
        assert!(!validate_hex_input("GG"));
        assert!(!validate_hex_input("ZZ 11"));
    }
}
