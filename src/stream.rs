
use super::error::{Error, ErrorCode, Result};
use super::decode::Decode;
use super::metadata::{MetadataHeader, StreamInfo};
use super::frame::{Frame};

pub struct Stream {
    pub stream_info: StreamInfo
}

impl Stream {
    pub fn new(reader: &mut Decode) -> Result<Self> {
        let magic = reader.read_u32()?;
        if magic != 0x664c6143 {
            return Err(Error::from_code(ErrorCode::WrongMagic))
        }
        let header = MetadataHeader::from_reader(reader)?;
        let stream_info = StreamInfo::from_reader(reader)?;
        if !header.last {
            loop {
                let header = MetadataHeader::from_reader(reader)?;
                header.skip_body(reader)?;
                if header.last {
                    break;
                }
            }
        }
        Ok(Stream { stream_info: stream_info })
    }

    pub fn decode_frames<F>(&self, reader: &mut Decode, mut sink: F) -> Result<()>
        where F: FnMut(&Frame) -> () {
        loop {
            let frame = match Frame::from_reader(reader, &self.stream_info)? {
                None => break,
                Some(frame) => frame
            };
            sink(&frame);
        }
        Ok(())
    }
}
