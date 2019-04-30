
use super::bits::BitRead;
use super::error::{Error, ErrorCode, Result};

pub enum MetadataType {
    StreamInfo,
    Reserved
}

impl From<u8> for MetadataType {
    fn from(u: u8) -> Self {
        match u {
            0 => MetadataType::StreamInfo,
            _ => MetadataType::Reserved
        }
    }
}

pub struct MetadataHeader {
    pub last: bool,
    pub r#type: MetadataType,
    pub length: usize
}

impl MetadataHeader {
    pub fn from_reader(reader: &mut BitRead) -> Result<Self> {
        let last   = reader.read_bool()?;
        let r#type = reader.read_u8_bits(7)?;
        let length = reader.read_u32_bits(24)?;
        let header = MetadataHeader {
            last: last,
            r#type: MetadataType::from(r#type),
            length: length as usize
        };
        Ok(header)
    }
}

#[derive(Debug)]
pub struct StreamInfo {
    pub min_block_size: usize,
    pub max_block_size: usize,
    pub min_frame_size: usize,
    pub max_frame_size: usize,
    pub sample_rate: usize,
    pub number_of_channels: usize,
    pub bits_per_sample: usize,
    pub total_samples: usize,
    pub signature: u128,
}

impl StreamInfo {
    pub fn from_reader(reader: &mut BitRead) -> Result<Self> {
        let min_block_size  = reader.read_u16()?;
        let max_block_size  = reader.read_u16()?;
        let min_frame_size  = reader.read_u32_bits(24)?;
        let max_frame_size  = reader.read_u32_bits(24)?;
        let sample_rate     = reader.read_u32_bits(20)?;
        let channels        = reader.read_u8_bits(3)?;
        let bits_per_sample = reader.read_u8_bits(5)?;
        let total_samples   = reader.read_u64_bits(36)?;
        let signature       = reader.read_u128()?;
        let stream_info = StreamInfo {
            min_block_size: min_block_size as usize,
            max_block_size: max_block_size as usize,
            min_frame_size: min_frame_size as usize,
            max_frame_size: max_frame_size as usize,
            sample_rate: sample_rate as usize,
            number_of_channels: (channels as usize) + 1,
            bits_per_sample: (bits_per_sample as usize) + 1,
            total_samples: total_samples as usize,
            signature: signature,
        };
        Ok(stream_info)
    }
}
