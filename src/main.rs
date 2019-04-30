
use std::fs::File;
use std::io::BufReader;

mod bits;
mod error;
mod stream;
mod metadata;
mod bitvec;

use bits::*;
use error::Result;
use stream::Stream; 

fn read() -> Result<()> {
    let file = File::open("/home/user/Desktop/starry.flac").unwrap();
    let mut buf = BufReader::new(file);
    let mut reader = BitReader::new(&mut buf);
    let stream = Stream::from_reader(&mut reader);
    Ok(())
}

fn main() {
    read().unwrap();
}
