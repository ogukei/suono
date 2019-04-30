
use super::error::{Error, ErrorCode, Result};
use super::metadata::StreamInfo;
use super::decode::Decode;

#[derive(Debug)]
pub struct FrameHeader {

}

impl FrameHeader {
    pub fn from_reader(reader: &mut Decode, stream_info: &StreamInfo) -> Result<Option<FrameHeader>> {
        reader.compute_crc8_begin();
        let sync_code = reader.read_u16_bits(14);
        match sync_code {
            Ok(sync_code) => {
                if sync_code != 0x3ffe {
                    return Err(Error::from_code(ErrorCode::InvalidSyncCode))
                }
            },
            Err(e) => return match e.kind() {
                std::io::ErrorKind::UnexpectedEof => Ok(None),
                _ => Err(Error::from(e))
            }
        };
        // parameters
        let _zero               = reader.read_bool()?;
        let blocking_strategy   = reader.read_u8_bits(1)?;
        let block_size_bits     = reader.read_u8_bits(4)?;
        let sample_rate_bits    = reader.read_u8_bits(4)?;
        let channel_assignment  = reader.read_u8_bits(4)?;
        let sample_size_in_bits = reader.read_u8_bits(3)?;
        let _reserved           = reader.read_bool()?;
        // skip utf-8 coded
        let mut v1: u32 = reader.read_u8()? as u32;
        while v1 >= 0b1100_0000 {
            reader.read_u8()?;
            v1 = (v1 << 1) & 0xff;
        }
        // variable block size
        let variable_block_size: Option<usize> = match block_size_bits {
            0b0110 => Some((reader.read_u8()? as usize) + 1),
            0b0111 => Some((reader.read_u16()? as usize) + 1),
            _ => None,
        };
        // variable sample rate
        let _ = match sample_rate_bits {
            0b1100 => reader.read_u8()? as u16,
            0b1101 => reader.read_u16()?,
            0b1110 => reader.read_u16()?,
            _ => 0u16,
        };
        // crc validate
        let actual_crc8 = reader.compute_crc8_end();
        let expected_crc8 = reader.read_u8()?;
        if actual_crc8 != expected_crc8 {
            return Err(Error::from_code(ErrorCode::InvalidFrameHeaderCrc))
        }
        // TODO:
        Ok(None)
    }
}