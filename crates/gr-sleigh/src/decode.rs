// SLEIGH .sla binary file decoder: zlib decompression + packed format parsing.

use crate::packed::{PackedReader, PackedError, ElementEvent, AttributeValue};

pub struct SlaDecoder {
    pub version: u32,
    pub big_endian: bool,
    pub alignment: u32,
    pub unique_base: u64,
    pub max_delay: u32,
    pub num_sections: u32,
    pub spaces: Vec<DecodedSpace>,
    pub source_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DecodedSpace {
    pub name: String,
    pub index: u32,
    pub size: u32,
    pub word_size: u32,
    pub space_type: String,
}

impl SlaDecoder {
    pub fn new() -> Self {
        Self {
            version: 0, big_endian: false, alignment: 1,
            unique_base: 0, max_delay: 0, num_sections: 0,
            spaces: Vec::new(), source_files: Vec::new(),
        }
    }

    pub fn decode_header(&mut self, reader: &mut PackedReader) -> Result<(), SlaDecodeError> {
        match reader.next_event()? {
            Some(ElementEvent::Start(_id)) => {
                while let Some((attr_id, value)) = reader.read_attribute()? {
                    match attr_id {
                        1 => if let AttributeValue::UnsignedInt(v) = value { self.version = v as u32; }
                        2 => if let AttributeValue::Bool(v) = value { self.big_endian = v; }
                        3 => if let AttributeValue::UnsignedInt(v) = value { self.alignment = v as u32; }
                        4 => if let AttributeValue::UnsignedInt(v) = value { self.unique_base = v; }
                        5 => if let AttributeValue::UnsignedInt(v) = value { self.max_delay = v as u32; }
                        6 => if let AttributeValue::UnsignedInt(v) = value { self.num_sections = v as u32; }
                        _ => {}
                    }
                }
                Ok(())
            }
            _ => Err(SlaDecodeError::InvalidFormat("expected sleigh element".into())),
        }
    }

    pub fn space_count(&self) -> usize { self.spaces.len() }
    pub fn source_file_count(&self) -> usize { self.source_files.len() }
}

impl Default for SlaDecoder {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, thiserror::Error)]
pub enum SlaDecodeError {
    #[error("packed format error: {0}")]
    Packed(#[from] PackedError),
    #[error("invalid format: {0}")]
    InvalidFormat(String),
    #[error("unsupported version: {0}")]
    UnsupportedVersion(u32),
    #[error("decompression failed")]
    DecompressFailed,
}

pub fn try_decompress_sla(data: &[u8]) -> Result<Vec<u8>, SlaDecodeError> {
    if data.len() < 2 {
        return Err(SlaDecodeError::InvalidFormat("too short".into()));
    }
    if data[0] == 0x78 && (data[1] == 0x01 || data[1] == 0x9C || data[1] == 0xDA) {
        return Err(SlaDecodeError::InvalidFormat("zlib compressed, decompression not yet implemented".into()));
    }
    Ok(data.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sla_decoder_default() {
        let decoder = SlaDecoder::new();
        assert_eq!(decoder.version, 0);
        assert_eq!(decoder.space_count(), 0);
    }

    #[test]
    fn try_decompress_uncompressed() {
        let data = vec![0x40, 0x01, 0x02, 0x03];
        let result = try_decompress_sla(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn try_decompress_compressed() {
        let data = vec![0x78, 0x9C, 0x00, 0x00];
        let result = try_decompress_sla(&data);
        assert!(result.is_err()); // no zlib support without feature
    }
}
