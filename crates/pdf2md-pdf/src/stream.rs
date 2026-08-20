use flate2::read::ZlibDecoder;
use std::io::Read;
use crate::security::{SecurityError, SecurityLimits};

pub struct StreamDecoder;

impl StreamDecoder {
    /// Safely decompresses a FlateDecode (Zlib) stream with strict decompression bomb checks
    pub fn decode_flate(compressed: &[u8], limits: &SecurityLimits) -> Result<Vec<u8>, SecurityError> {
        if compressed.is_empty() {
            return Ok(Vec::new());
        }

        let max_allowed_bytes = limits.max_decompressed_stream_bytes;
        let max_ratio = limits.max_decompression_ratio;

        let mut decoder = ZlibDecoder::new(compressed);
        let mut output = Vec::new();
        let mut chunk = [0u8; 8192];
        let mut total_read = 0;

        loop {
            match decoder.read(&mut chunk) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    total_read += n;

                    if total_read > max_allowed_bytes {
                        return Err(SecurityError::StreamSizeExceeded(total_read, max_allowed_bytes));
                    }

                    let current_ratio = (total_read as f32) / (compressed.len().max(1) as f32);
                    if current_ratio > max_ratio && total_read > 1024 * 1024 {
                        return Err(SecurityError::DecompressionBomb(current_ratio, max_ratio));
                    }

                    output.extend_from_slice(&chunk[..n]);
                }
                Err(e) => {
                    return Err(SecurityError::LimitExceeded(format!(
                        "Flate decompression error: {}",
                        e
                    )));
                }
            }
        }

        Ok(output)
    }

    /// Decodes an ASCIIHexDecode stream
    pub fn decode_ascii_hex(input: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let mut out = Vec::with_capacity(input.len() / 2);
        let mut high_nibble: Option<u8> = None;

        for &b in input {
            if b == b'>' {
                break;
            }
            if b.is_ascii_whitespace() {
                continue;
            }

            let val = match b {
                b'0'..=b'9' => b - b'0',
                b'a'..=b'f' => b - b'a' + 10,
                b'A'..=b'F' => b - b'A' + 10,
                _ => {
                    return Err(SecurityError::LimitExceeded(format!(
                        "Invalid character in ASCIIHex stream: {}",
                        b as char
                    )));
                }
            };

            if let Some(high) = high_nibble.take() {
                out.push((high << 4) | val);
            } else {
                high_nibble = Some(val);
            }
        }

        if let Some(high) = high_nibble {
            out.push(high << 4);
        }

        Ok(out)
    }
}

pub fn decode_stream(compressed: &[u8], filter: Option<&str>, limits: &SecurityLimits) -> Result<Vec<u8>, SecurityError> {
    match filter {
        Some("FlateDecode") => StreamDecoder::decode_flate(compressed, limits),
        Some("ASCIIHexDecode") => StreamDecoder::decode_ascii_hex(compressed),
        _ => Ok(compressed.to_vec()),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    #[test]
    pub fn test_ascii_hex() {
        let input = b"48656C6C6F>";
        let decoded = StreamDecoder::decode_ascii_hex(input).unwrap();
        assert_eq!(decoded, b"Hello");
    }

    #[test]
    pub fn test_decompression_bomb_rejected() {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
        let zeros = vec![0u8; 2 * 1024 * 1024]; // 2MB zeros
        encoder.write_all(&zeros).unwrap();
        let compressed = encoder.finish().unwrap();

        let mut limits = SecurityLimits::default();
        limits.max_decompressed_stream_bytes = 1024 * 1024; // 1 MB limit

        let res = StreamDecoder::decode_flate(&compressed, &limits);
        assert!(res.is_err(), "Decompression bomb must be rejected");
        match res {
            Err(SecurityError::StreamSizeExceeded(_, _)) => {}
            other => panic!("Expected StreamSizeExceeded error, got {:?}", other),
        }
    }
}
