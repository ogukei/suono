
use std::io::Read;
use std::io::Result;

pub trait BitRead {
    fn read_bool(&mut self) -> Result<bool>;
    fn read_u8(&mut self) -> Result<u8>;
    fn read_u16(&mut self) -> Result<u16>;
    fn read_u32(&mut self) -> Result<u32>;
    fn read_u64(&mut self) -> Result<u64>;
    fn read_u128(&mut self) -> Result<u128>;
    fn read_u8_bits(&mut self, n: usize) -> Result<u8>;
    fn read_u16_bits(&mut self, n: usize) -> Result<u16>;
    fn read_u32_bits(&mut self, n: usize) -> Result<u32>;
    fn read_u64_bits(&mut self, n: usize) -> Result<u64>;
}

pub struct BitReader<'a, Source> {
    source: &'a mut Source,
    queue: u64,
    queue_count: isize
}

impl<'a, Source: Read> BitReader<'a, Source> {
    pub fn new(source: &'a mut Source) -> Self {
        BitReader {
            source: source,
            queue: 0,
            queue_count: 0
        }
    }

    #[inline]
    fn read_value(&mut self, n: isize) -> Result<u64> {
        assert!(n <= 64 && n >= 0);
        let result: u64;
        let n_bits = n - self.queue_count;
        if n_bits > 0 {
            // consume some bits from the source
            let n_bytes = ((n_bits - 1) >> 3) + 1;
            let mut array: [u8; 8] = [0u8; 8];
            let offset = (8 - n_bytes) as usize;
            self.source.read_exact(&mut array[offset..])?;
            // interpret bits as u64
            let loaded = u64::from_be_bytes(array);
            let dequeued = self.queue.checked_shl(n_bits as u32).unwrap_or(0);
            let remaining = (8 - (n_bits & 7)) & 7;
            result = dequeued | (loaded >> remaining);
            self.queue = loaded & ((1 << remaining) - 1);
            self.queue_count = remaining;
        } else {
            // use internal cache
            let remaining = -n_bits;
            let queue = self.queue;
            result = queue >> remaining;
            self.queue = queue & ((1 << remaining) - 1);
            self.queue_count = remaining;
        }
        Ok(result)
    }
}

impl<'a, Source: Read> BitRead for BitReader<'a, Source> {
    fn read_bool(&mut self) -> Result<bool> {
        let value = self.read_value(1)?;
        Ok((value & 1u64) == 1u64)
    }

    fn read_u8_bits(&mut self, n: usize) -> Result<u8> {
        assert!(n <= 8);
        let value = self.read_value(n as isize)?;
        Ok((value & 0xffu64) as u8)
    }

    fn read_u8(&mut self) -> Result<u8> {
        let value = self.read_value(8)?;
        Ok((value & 0xffu64) as u8)
    }

    fn read_u16_bits(&mut self, n: usize) -> Result<u16> {
        assert!(n <= 16);
        let value = self.read_value(n as isize)?;
        Ok((value & 0xffffu64) as u16)
    }

    fn read_u16(&mut self) -> Result<u16> {
        let value = self.read_value(16)?;
        Ok((value & 0xffffu64) as u16)
    }

    fn read_u32_bits(&mut self, n: usize) -> Result<u32> {
        assert!(n <= 32);
        let value = self.read_value(n as isize)?;
        Ok((value & 0xffffffffu64) as u32)
    }

    fn read_u32(&mut self) -> Result<u32> {
        let value = self.read_value(32)?;
        Ok((value & 0xffffffffu64) as u32)
    }

    fn read_u64_bits(&mut self, n: usize) -> Result<u64> {
        assert!(n <= 64);
        self.read_value(n as isize)
    }

    fn read_u64(&mut self) -> Result<u64> {
        self.read_value(64)
    }

    fn read_u128(&mut self) -> Result<u128> {
        let mut value: u128 = 0;
        value |= (self.read_value(64)? as u128) << 64;
        value |= self.read_value(64)? as u128;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flac_magic() {
        let mut bytes: &[u8] = &[0x66, 0x4c, 0x61, 0x43, 0, 0, 0x22];
        let mut reader = BitReader::new(&mut bytes);
        assert_eq!(reader.read_u32().unwrap(), 0x664c6143);
        assert_eq!(reader.read_u16().unwrap(), 0);
        assert_eq!(reader.read_u8().unwrap(), 0x22);
    }

    #[test]
    fn test_cross_boundary() {
        let mut bytes: &[u8] = &[0b10110110, 0b11001100, 0b11110110, 0b11001001,
                                 0b10001001, 0b11101101, 0b01001000, 0b01011001, 0b01011001];
        let mut reader = BitReader::new(&mut bytes);
        assert_eq!(reader.read_u64_bits(62).unwrap(), 
                   0b10110110110011001111011011001001100010011110110101001000010110);
        assert_eq!(reader.read_u64_bits(10).unwrap(), 
                   0b0101011001);
    }

    #[test]
    fn test_boundary() {
        let mut bytes: &[u8] = &[0b10110110, 0b11001100, 0b11110110, 0b11001001,
                                 0b10001001, 0b11101101, 0b01001000, 0b01011001, 0b01011001];
        let mut reader = BitReader::new(&mut bytes);
        assert_eq!(reader.read_u64().unwrap(), 
                   0b1011011011001100111101101100100110001001111011010100100001011001);
        assert_eq!(reader.read_u8().unwrap(), 
                   0b01011001);
    }

    #[test]
    fn test_singular() {
        let mut bytes: &[u8] = &[0b10110110, 0b11001100, 0b11110110, 0b11001001,
                                 0b10001001, 0b11101101, 0b01001000, 0b01011001, 0b11010110];
        let mut reader = BitReader::new(&mut bytes);
        assert_eq!(reader.read_u64_bits(62).unwrap(), 
                   0b10110110110011001111011011001001100010011110110101001000010110);
        assert_eq!(reader.read_u64_bits(1).unwrap(), 0b0);
        assert_eq!(reader.read_u64_bits(1).unwrap(), 0b1);
        assert_eq!(reader.read_u64_bits(1).unwrap(), 0b1);
        assert_eq!(reader.read_u64_bits(1).unwrap(), 0b1);
        assert_eq!(reader.read_u64_bits(1).unwrap(), 0b0);
        assert_eq!(reader.read_u64_bits(1).unwrap(), 0b1);
        assert_eq!(reader.read_u64_bits(1).unwrap(), 0b0);
        assert_eq!(reader.read_u64_bits(1).unwrap(), 0b1);
        assert_eq!(reader.read_u64_bits(1).unwrap(), 0b1);
        assert_eq!(reader.read_u64_bits(1).unwrap(), 0b0);
    }
}
