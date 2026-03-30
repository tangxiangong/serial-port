use crate::core::checksum::ChecksumKind;
use crate::core::records::DisplayMode;
use anyhow::{Context, Result, anyhow};

pub fn format_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("0x{:02X}", byte))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn format_dec(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| byte.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn sanitize_hex(input: &str) -> String {
    input
        .chars()
        .filter(|ch| ch.is_ascii_hexdigit())
        .collect::<String>()
        .to_uppercase()
}

pub fn reformat_hex(input: &str) -> String {
    let cleaned = sanitize_hex(input);
    let mut chars = cleaned.chars().collect::<Vec<_>>();
    if chars.len() % 2 != 0 {
        chars.insert(chars.len().saturating_sub(1), '0');
    }
    chars
        .chunks(2)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn parse_hex(input: &str) -> Result<Vec<u8>> {
    let cleaned = sanitize_hex(input);
    if cleaned.is_empty() {
        return Ok(Vec::new());
    }
    if !cleaned.len().is_multiple_of(2) {
        return Err(anyhow!("HEX 输入必须是偶数长度"));
    }
    let bytes = cleaned
        .as_bytes()
        .chunks(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).context("HEX 输入包含非法字符")?;
            u8::from_str_radix(pair, 16).context("HEX 解析失败")
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(bytes)
}

pub fn parse_dec(input: &str) -> Result<Vec<u8>> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    if trimmed.contains(' ') {
        return trimmed
            .split_whitespace()
            .map(|part| {
                part.parse::<u8>()
                    .with_context(|| format!("十进制字节 `{part}` 超出范围"))
            })
            .collect::<Result<Vec<_>>>();
    }

    let value = trimmed
        .parse::<u64>()
        .with_context(|| format!("无法解析十进制值 `{trimmed}`"))?;
    let mut bytes = value.to_be_bytes().to_vec();
    while bytes.len() > 1 && bytes.first() == Some(&0) {
        bytes.remove(0);
    }
    Ok(bytes)
}

pub fn encode_payload(
    text: &str,
    display: DisplayMode,
    line_ending: &str,
    checksum: Option<ChecksumKind>,
) -> Result<Vec<u8>> {
    let mut bytes = match display {
        DisplayMode::Hex => parse_hex(text)?,
        DisplayMode::Ascii => {
            let mut encoded = text.as_bytes().to_vec();
            encoded.extend_from_slice(line_ending.as_bytes());
            encoded
        }
        DisplayMode::Dec => parse_dec(text)?,
    };

    if let Some(kind) = checksum {
        bytes.extend_from_slice(&kind.apply(&bytes));
    }
    Ok(bytes)
}

pub fn render_payload(data: &[u8], display: DisplayMode) -> String {
    match display {
        DisplayMode::Hex => format_hex(data),
        DisplayMode::Ascii => String::from_utf8_lossy(data).into_owned(),
        DisplayMode::Dec => format_dec(data),
    }
}

#[cfg(test)]
mod tests {
    use super::{encode_payload, parse_dec, parse_hex, reformat_hex, render_payload};
    use crate::core::records::DisplayMode;

    #[test]
    fn hex_reformat_is_stable() {
        assert_eq!(reformat_hex("123"), "12 03");
    }

    #[test]
    fn parse_hex_works() {
        assert_eq!(parse_hex("AA 55").unwrap(), vec![0xAA, 0x55]);
    }

    #[test]
    fn parse_dec_supports_scalar_and_bytes() {
        assert_eq!(parse_dec("255").unwrap(), vec![255]);
        assert_eq!(parse_dec("1 2 3").unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn ascii_encode_appends_line_ending() {
        let bytes = encode_payload("AT", DisplayMode::Ascii, "\r\n", None).unwrap();
        assert_eq!(bytes, b"AT\r\n");
    }

    #[test]
    fn render_payload_decodes_utf8() {
        assert_eq!(render_payload(b"ok", DisplayMode::Ascii), "ok");
    }
}
