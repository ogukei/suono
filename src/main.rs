
extern crate itertools;
extern crate hound;

mod bits;
mod error;
mod stream;
mod metadata;
mod bitvec;
mod frame;
mod crc;
mod decode;

use std::fs::File;
use std::io::BufReader;

use error::Result;
use bits::BitReader;
use decode::DecodingReadProxy;
use frame::Frame;
use stream::Stream;

// a usage example converting .flac to .wav
fn decode_to_wav() -> Result<()> {
    let file = File::open("input.flac").unwrap();
    let mut buf = BufReader::new(file);
    // some setup for our decoder
    let mut proxy = DecodingReadProxy::new(&mut buf);
    let mut reader = BitReader::new(&mut proxy);
    // start reading FLAC stream
    let stream = Stream::new(&mut reader)?;
    let info = stream.stream_info;
    println!("{:?}", info);
    // writer setup
    let spec = hound::WavSpec {
        channels: info.number_of_channels as u16,
        sample_rate: info.sample_rate as u32,
        bits_per_sample: info.bits_per_sample as u16,
        sample_format: hound::SampleFormat::Int,
    };
    let writer = &mut hound::WavWriter::create("output.wav", spec).unwrap();
    // frame processing
    let frame_sink = |frame: &Frame| {
        match frame.blocks.len() {
            2 => {
                // stereo
                let left = &frame.blocks[0];
                let right = &frame.blocks[1];
                for sample in itertools::interleave(left, right) {
                    writer.write_sample(*sample).unwrap();
                }
            },
            1 => {
                // monaural
                for sample in &frame.blocks[0] {
                    writer.write_sample(*sample).unwrap();
                }
            },
            _ => unreachable!()
        }
    };
    println!("decoding frames...");
    stream.decode_frames(&mut reader, frame_sink)?;
    println!("done");
    Ok(())
}

fn main() {
    decode_to_wav().unwrap();
}
