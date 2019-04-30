
use super::bits::BitRead;
use super::error::{Error, ErrorCode, Result};

use super::metadata::{MetadataHeader, StreamInfo};

pub struct Stream {

}

impl Stream {
    pub fn from_reader(reader: &mut BitRead) -> Result<Self> {
        let magic = reader.read_u32()?;
        if magic != 0x664c6143 {
            return Err(Error::from_code(ErrorCode::WrongMagic))
        }
        let header = MetadataHeader::from_reader(reader);
        let stream_info = StreamInfo::from_reader(reader);
        println!("{:?}", stream_info);
        Ok(Stream { })
    }
}
