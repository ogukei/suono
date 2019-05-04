
use std::io;
use std::io::Read;
use super::crc::{Hasher, HasherCrc8, HasherCrc16Buypass};
use super::bits::{BitRead, BitReader};

pub trait Decode: BitRead + DecodingRead {
    fn decode_rice(&mut self, parameter: usize) -> io::Result<i32>;
}

pub trait DecodingRead {
    fn compute_crc8_begin(&mut self);
    fn compute_crc8_end(&mut self) -> u8;
    fn compute_crc16_begin(&mut self);
    fn compute_crc16_end(&mut self) -> u16;
}

pub struct DecodingReadProxy<'a> {
    underlying: &'a mut Read,
    crc8: HasherCrc8,
    crc16: HasherCrc16Buypass,
    computing_crc8: bool,
    computing_crc16: bool
}

impl<'a> DecodingReadProxy<'a> {
    pub fn new(reader: &'a mut Read) -> Self {
        DecodingReadProxy {
            underlying: reader,
            crc8: HasherCrc8::new(),
            crc16: HasherCrc16Buypass::new(),
            computing_crc8: false,
            computing_crc16: false
        }
    }
}

impl<'a> Read for DecodingReadProxy<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.underlying.read(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let result = self.underlying.read_exact(buf);
        if self.computing_crc8 {
            self.crc8.hash(buf);
        }
        if self.computing_crc16 {
            self.crc16.hash(buf);
        }
        result
    }
}

impl<'a> DecodingRead for DecodingReadProxy<'a> {
    fn compute_crc8_begin(&mut self) {
        self.computing_crc8 = true;
        self.crc8.reset()
    }

    fn compute_crc8_end(&mut self) -> u8 {
        self.computing_crc8 = false;
        self.crc8.state()
    }

    fn compute_crc16_begin(&mut self) {
        self.computing_crc16 = true;
        self.crc16.reset()
    }

    fn compute_crc16_end(&mut self) -> u16 {
        self.computing_crc16 = false;
        self.crc16.state()
    }
}

// BitReader Extension
impl<'a, Source: DecodingRead> DecodingRead for BitReader<'a, Source> {
    fn compute_crc8_begin(&mut self) {
        self.source.compute_crc8_begin()
    }

    fn compute_crc8_end(&mut self) -> u8 {
        self.source.compute_crc8_end()
    }

    fn compute_crc16_begin(&mut self) {
        self.source.compute_crc16_begin()
    }

    fn compute_crc16_end(&mut self) -> u16 {
        self.source.compute_crc16_end()
    }
}

impl<'a, Source: Read + DecodingRead> Decode for BitReader<'a, Source> {
    // Rice Decoding
    fn decode_rice(&mut self, parameter: usize) -> io::Result<i32> {
        // unary decoding
        let msb: u32 = self.read_unary()?;
        let lsb = self.read_u32_bits(parameter)?;
        let v = ((msb << parameter) | lsb) as i32;
        // convert to signed (zig-zag decoding)
        Ok((v >> 1) ^ -(v & 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rice() {
        let mut bytes: &[u8] = &[0b1000_1001, 0b1010_1011, 
                                 0b1100_0000, 0b1010_0000, 0b0000_0101];
        let mut proxy = DecodingReadProxy::new(&mut bytes);
        let mut reader = BitReader::new(&mut proxy);
        assert_eq!(reader.decode_rice(3).unwrap(), 0);
        assert_eq!(reader.decode_rice(3).unwrap(), -1);
        assert_eq!(reader.decode_rice(3).unwrap(), 1);
        assert_eq!(reader.decode_rice(3).unwrap(), -2);
        assert_eq!(reader.decode_rice(3).unwrap(), 2);
        // (4 << 3) + 1
        assert_eq!(reader.decode_rice(3).unwrap(), 17);
        // -((9 << 2) + 1)
        assert_eq!(reader.decode_rice(2).unwrap(), -19);
    }
}
