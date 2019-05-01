
use super::decode::Decode;
use super::error::{Error, ErrorCode, Result};
use super::metadata::{MetadataHeader, StreamInfo};
use super::frame::{Frame};

pub struct Stream {

}

impl Stream {
    pub fn from_reader(reader: &mut Decode) -> Result<Self> {
        let magic = reader.read_u32()?;
        if magic != 0x664c6143 {
            return Err(Error::from_code(ErrorCode::WrongMagic))
        }
        let header = MetadataHeader::from_reader(reader)?;
        let stream_info = StreamInfo::from_reader(reader)?;
        println!("{:?}", stream_info);
        if !header.last {
            loop {
                let header = MetadataHeader::from_reader(reader)?;
                header.skip_body(reader);
                println!("{:?}", header);
                if header.last {
                    break;
                }
            }
        }
        loop {
            let frame = match Frame::from_reader(reader, &stream_info)? {
                None => break,
                Some(frame) => frame
            };
            println!("{:?}", frame);
        }
        Ok(Stream { })
    }
}
