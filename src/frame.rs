
use std::io;
use super::error::{Error, ErrorCode, Result};
use super::metadata::StreamInfo;
use super::decode::Decode;
use super::bitvec::Bitvec;

#[derive(Debug)]
pub struct Frame {
    header: FrameHeader
}

impl Frame {
    pub fn from_reader(reader: &mut Decode, stream_info: &StreamInfo) -> Result<Option<Self>> {
        reader.compute_crc16_begin();
        let header = match FrameHeader::from_reader(reader, stream_info)? {
            None => {
                reader.compute_crc16_end();
                return Ok(None)
            },
            Some(header) => header
        };
        let mut vec: Vec<i32> = Vec::new();
        match header.channel_assignment {
            ChannelAssignment::Independent(num_channels) => {
                for index in 0..num_channels {
                    let subframe = Subframe::from_reader(reader, header.sample_size, header.block_size)?;
                    subframe.decode(reader, &mut vec)?;
                }
            },
            _ => ()
        }
        println!("{:?}", vec);
        let frame = Frame { header: header };
        reader.align_to_byte();
        let actual_crc16 = reader.compute_crc16_end();
        let expected_crc16 = reader.read_u16()?;
        if actual_crc16 != expected_crc16 {
            return Err(Error::from_code(ErrorCode::FrameCrcMismatch))
        }
        Ok(Some(frame))
    }
}

#[derive(Debug)]
pub struct FrameHeader {
    pub sample_size: usize,
    pub block_size: usize,
    pub channel_assignment: ChannelAssignment
}

impl FrameHeader {
    pub fn from_reader(reader: &mut Decode, stream_info: &StreamInfo) -> Result<Option<Self>> {
        reader.compute_crc8_begin();
        let sync_code = reader.read_u16_bits(14);
        match sync_code {
            Ok(sync_code) => {
                if sync_code != 0x3ffe {
                    return Err(Error::from_code(ErrorCode::FrameOutOfSync))
                }
            },
            Err(e) => return match e.kind() {
                io::ErrorKind::UnexpectedEof => Ok(None),
                _ => Err(Error::from(e))
            }
        };
        // parameters
        let _zero             = reader.read_bool()?;
        let blocking_strategy = reader.read_u8_bits(1)?;
        let block_size_bits   = reader.read_u8_bits(4)?;
        let sample_rate_bits  = reader.read_u8_bits(4)?;
        let channel_bits      = reader.read_u8_bits(4)?;
        let sample_size_bits  = reader.read_u8_bits(3)?;
        let _reserved         = reader.read_bool()?;
        // skip utf-8 coded
        let mut v1: u32 = reader.read_u8()? as u32;
        while v1 >= 0b1100_0000 {
            reader.read_u8()?;
            v1 = (v1 << 1) & 0xff;
        }
        // variable block size
        let variable_block_size: Option<usize> = match block_size_bits {
            0b0110 => reader.read_u8()
                .map(|x| (x as usize) + 1)
                .map(|x| Some(x))?,
            0b0111 => reader.read_u16()
                .map(|x| (x as usize) + 1)
                .map(|x| Some(x))?,
            _ => None
        };
        // variable sample rate
        let _ = match sample_rate_bits {
            0b1100 => reader.read_u8()? as u16,
            0b1101 => reader.read_u16()?,
            0b1110 => reader.read_u16()?,
            _ => 0u16
        };
        // crc validate
        let actual_crc8 = reader.compute_crc8_end();
        let expected_crc8 = reader.read_u8()?;
        if actual_crc8 != expected_crc8 {
            return Err(Error::from_code(ErrorCode::FrameHeaderCrcMismatch))
        }
        let sample_size = |n: u8| -> Option<usize> {
            let size = match n {
                0b000 => stream_info.bits_per_sample as usize,
                0b001 => 8,
                0b010 => 12,
                0b100 => 16,
                0b101 => 20,
                0b110 => 24,
                _ => return None
            };
            Some(size)
        };
        let block_size = |n: u8| -> Option<usize> {
            let size = match n {
                0b0001 => 192,
                0b0010..=0b0101 => 576 * (1 << ((n as i32)-2)),
                0b0110 => variable_block_size?,
                0b0111 => variable_block_size?,
                0b1000..=0b1111 => 256 * (1 << ((n as i32)-8)),
                _ => return None
            };
            Some(size)
        };
        let header = FrameHeader {
            sample_size: sample_size(sample_size_bits)
                .ok_or(Error::from_code(ErrorCode::FrameSampleSizeUnknown))?,
            block_size: block_size(block_size_bits)
                .ok_or(Error::from_code(ErrorCode::FrameBlockSizeUnknown))?,
            channel_assignment: ChannelAssignment::parse(channel_bits)
                .ok_or(Error::from_code(ErrorCode::FrameChannelAssignmentUnknown))?
        };
        Ok(Some(header))
    }
}

// SUBFRAME
#[derive(Debug)]
struct Subframe {
    method: PredictionMethod,
    sample_size: usize,
    num_samples: usize
}

impl Subframe {
    fn from_reader(reader: &mut Decode, sample_size: usize, block_size: usize) -> Result<Self> {
        let header = SubframeHeader::from_reader(reader)?;
        let sample_size = sample_size - header.wasted_bits_per_sample;
        let subframe = Subframe { 
            method: header.method,
            sample_size: sample_size,
            num_samples: block_size
        };
        Ok(subframe)
    }

    fn decode(&self, reader: &mut Decode, vec: &mut Vec<i32>) -> Result<()> {
        match self.method {
            PredictionMethod::Constant => self.decode_constant(reader, vec),
            _ => unreachable!()
        }
    }

    fn decode_constant(&self, reader: &mut Decode, vec: &mut Vec<i32>) -> Result<()> {
        let bps = self.sample_size;
        let num_samples = self.num_samples;
        let sample = sign_extend(reader.read_u64_bits(bps)?, bps) as i32;
        let offset = vec.len();
        vec.resize(offset + num_samples, sample);
        Ok(())
    }
}

// SUBFRAME_HEADER
#[derive(Debug, Clone, Copy)]
enum PredictionMethod {
    Constant,
    Verbatim,
    Fixed(usize),
    Fir(usize)
}

impl PredictionMethod {
    fn parse(n: u8) -> Option<Self> {
        let method = match n {
            0b00_0000 => PredictionMethod::Constant,
            0b00_0001 => PredictionMethod::Verbatim,
            0b00_1000..=0b00_1111 => PredictionMethod::Fixed((n & 0b00_0111) as usize),
            0b10_0000..=0b11_1111 => PredictionMethod::Fir(((n & 0b01_1111) as usize) + 1),
            _ => return None
        };
        Some(method)
    }
}

#[derive(Debug)]
struct SubframeHeader {
    pub method: PredictionMethod,
    pub wasted_bits_per_sample: usize
}

impl SubframeHeader {
    pub fn from_reader(reader: &mut Decode) -> Result<Self> {
        // Zero bit padding, to prevent sync-fooling string of 1s
        let zero = reader.read_bool()?;
        if zero {
            return Err(Error::from_code(ErrorCode::SubframeOutOfSync))
        }
        // Subframe type
        let method = PredictionMethod::parse(reader.read_u8_bits(6)?)
            .ok_or(Error::from_code(ErrorCode::SubframeReservedType))?;
        // 'Wasted bits-per-sample' flag
        let wasted_flag = reader.read_bool()?;
        let mut wasted_bits_per_sample: usize = 0;
        if wasted_flag {
            loop {
                wasted_bits_per_sample += 1;
                if reader.read_bool()? {
                    break;
                }
            }
        }
        let header = SubframeHeader {
            method: method,
            wasted_bits_per_sample: wasted_bits_per_sample
        };
        Ok(header)
    }
}

#[derive(Debug)]
pub enum ChannelAssignment {
    Independent(usize),
    LeftSideStereo,
    RightSideStereo,
    MidSideStereo
}

impl ChannelAssignment {
    pub fn parse(n: u8) -> Option<Self> {
        let assignment = match n {
            0b0000..=0b0111 => ChannelAssignment::Independent((n as usize) + 1),
            0b1000 => ChannelAssignment::LeftSideStereo,
            0b1001 => ChannelAssignment::RightSideStereo,
            0b1010 => ChannelAssignment::MidSideStereo,
            _ => return None
        };
        Some(assignment)
    }

    pub fn number_of_channels(&self) -> usize {
        match *self {
            ChannelAssignment::Independent(num) => num,
            ChannelAssignment::LeftSideStereo => 2,
            ChannelAssignment::RightSideStereo => 2,
            ChannelAssignment::MidSideStereo => 2
        }
    }
}

fn sign_extend(x: u64, n: usize) -> i64 {
    let m = 64 - n;
    ((x << m) as i64) >> m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_extend() {
        assert_eq!(sign_extend(0b110, 3), -2);
        assert_eq!(sign_extend(0b10110011, 8), -77);
        assert_eq!(sign_extend(0b001, 3), 1);
        assert_eq!(sign_extend(0b00110011, 8), 51);
    }
}
