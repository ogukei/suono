
use std::fs::File;
use std::io::BufReader;

mod bits;
mod error;
mod stream;
mod metadata;
mod bitvec;
mod frame;
mod crc;
mod decode;

use bits::*;
use error::Result;
use stream::Stream;
use decode::DecodingReadProxy;

fn read() -> Result<()> {
    let file = File::open("/home/user/Desktop/starry.flac").unwrap();
    let mut buf = BufReader::new(file);
    let mut proxy = DecodingReadProxy::new(&mut buf);
    let mut reader = BitReader::new(&mut proxy);
    let stream = Stream::from_reader(&mut reader)?;
    Ok(())
}

fn main() {
    read().unwrap();
}
