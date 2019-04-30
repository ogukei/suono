
use std::io::Result;
use std::io::Read;

#[derive(PartialEq, Debug)]
pub enum BitvecBlock {
    Bytes(Vec<u8>),
    Bits(u8, usize)
}

#[derive(Debug)]
pub struct Bitvec {
    pub blocks: Vec<BitvecBlock>
}

impl Bitvec {
    pub fn new() -> Self {
        Bitvec {
            blocks: vec![]
        }
    }

    pub fn write_bits(&mut self, u: u8, n: usize) {
        if n == 0 {
            return
        }
        match (n, self.blocks.last_mut()) {
            (8, Some(BitvecBlock::Bytes(vec))) => vec.push(u),
            (8, None) => self.blocks.push(BitvecBlock::Bytes(vec![u])),
            // Shrink
            (_, Some(BitvecBlock::Bits(w, q))) if (n + *q) < 8 => {
                let cw = *w;
                let cq = *q;
                let mask: u8 = ((1u16 << n) - 1) as u8;
                let v = (cw << n) | (u & mask);
                *w = v;
                *q = n + cq;
            },
            // Shrink Fit
            (_, Some(BitvecBlock::Bits(w, q))) if (n + *q) == 8 => {
                let w = *w;
                let mask: u8 = ((1u16 << n) - 1) as u8;
                let v = (w << n) | (u & mask);
                self.blocks.pop();
                match self.blocks.last_mut() {
                    Some(BitvecBlock::Bytes(vec)) => vec.push(v),
                    _ => self.blocks.push(BitvecBlock::Bytes(vec![v]))
                }
            },
            // Shrink Overflow
            (_, Some(BitvecBlock::Bits(w, q))) if (n + *q) > 8 => {
                let w = *w;
                let q = *q;
                let fill = (8 - q) & 7;
                let overflow = (n + q) & 7;
                let msb = (w << fill) | (u >> overflow);
                let lsb = u & (((1u16 << overflow) - 1) as u8);
                self.blocks.pop();
                match self.blocks.last_mut() {
                    Some(BitvecBlock::Bytes(vec)) => vec.push(msb),
                    _ => self.blocks.push(BitvecBlock::Bytes(vec![msb]))
                };
                self.blocks.push(BitvecBlock::Bits(lsb, overflow))
            },
            // Append
            _ => self.blocks.push(BitvecBlock::Bits(u, n))
        }
    }

    pub fn write_bytes(&mut self, reader: &mut Read, n: usize) -> Result<()> {
        if n == 0 {
            return Ok(())
        }
        match self.blocks.last_mut() {
            Some(BitvecBlock::Bytes(vec)) => {
                let offset = vec.len();
                let new_len = offset + n;
                vec.resize(new_len, 0);
                let slice = &mut vec[offset..];
                reader.read_exact(slice)?;
                Ok(())
            },
            _ => {
                let mut vec: Vec<u8> = Vec::new();
                vec.resize(n, 0);
                let slice = &mut vec[..];
                reader.read_exact(slice)?;
                self.blocks.push(BitvecBlock::Bytes(vec));
                Ok(())
            }
        }
    }
}

impl PartialEq for Bitvec {
    fn eq(&self, other: &Bitvec) -> bool {
        self.blocks == other.blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_bytes() {
        let mut bytes: &[u8] = &[0x66, 0x4c, 0x61, 0x43, 0, 0, 0x22];
        let mut vec = Bitvec::new();
        vec.write_bytes(&mut bytes, 4).unwrap();
        vec.write_bytes(&mut bytes, 2).unwrap();
        vec.write_bytes(&mut bytes, 1).unwrap();
        assert_eq!(vec, Bitvec { 
            blocks: vec![
                BitvecBlock::Bytes(vec![0x66, 0x4c, 0x61, 0x43, 0, 0, 0x22])
            ]
        })
    }

    #[test]
    fn test_write_bits() {
        let mut bytes: &[u8] = &[0b10110110, 0b11001100];
        let mut vec = Bitvec::new();
        vec.write_bits(0b10110, 5);
        vec.write_bytes(&mut bytes, 2).unwrap();
        vec.write_bits(0b11110110, 8);
        assert_eq!(vec, Bitvec {
            blocks: vec![
                BitvecBlock::Bits(0b10110, 5),
                BitvecBlock::Bytes(vec![0b10110110, 0b11001100, 0b11110110])
            ]
        })
    }

    #[test]
    fn test_write_overflow() {
        let mut vec = Bitvec::new();
        vec.write_bits(0b11110110, 8);
        vec.write_bits(0b101101, 6);
        vec.write_bits(0b111100, 6);
        assert_eq!(vec, Bitvec {
            blocks: vec![
                BitvecBlock::Bytes(vec![0b11110110, 0b10110111]),
                BitvecBlock::Bits(0b1100, 4)
            ]
        })
    }

    #[test]
    fn test_write_overflow_twice() {
        let mut vec = Bitvec::new();
        vec.write_bits(0b11, 2);
        vec.write_bits(0b01101101, 8);
        vec.write_bits(0b10111100, 8);
        assert_eq!(vec, Bitvec {
            blocks: vec![
                BitvecBlock::Bytes(vec![0b11011011, 0b01101111]),
                BitvecBlock::Bits(0, 2)
            ]
        })
    }
}
