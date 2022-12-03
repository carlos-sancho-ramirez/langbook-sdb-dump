use std::fs::File;
use std::io::Bytes;
use crate::file_utils;
use file_utils::ReadError;

pub struct InputBitStream<'a> {
    bytes: &'a mut Bytes<File>,
    buffer: u8,
    remaining: u32
}

impl<'a> InputBitStream<'a> {
    fn read_boolean(&mut self) -> Result<bool, ReadError> {
        if self.remaining == 0 {
            self.buffer = file_utils::read_u8(self.bytes)?;
            self.remaining = 8;
        }

        let result = (self.buffer & 1) != 0;
        self.buffer >>= 1;
        self.remaining -= 1;
        Ok(result)
    }

    pub fn read_symbol<S, T : HuffmanTable<S>>(&mut self, table: &T) -> Result<S, ReadError> {
        if table.symbols_with_bits(0) > 0 {
            Ok(table.get_symbol(0, 0)?)
        }
        else {
            let mut value = 0u32;
            let mut base = 0u32;
            let mut bits = 1u32;

            loop {
                value <<= 1;
                if self.read_boolean()? {
                    value += 1;
                }

                base <<= 1;
                let level_length = table.symbols_with_bits(bits);
                let level_index = value - base;
                if level_index < level_length {
                    return Ok(table.get_symbol(bits, level_index)?);
                }

                base += level_length;
                bits += 1;
            }
        }
    }
}

impl<'a> From<&'a mut Bytes<File>> for InputBitStream<'a> {
    fn from(bytes: &'a mut Bytes<File>) -> InputBitStream<'a> {
        InputBitStream {
            bytes,
            buffer: 0,
            remaining: 0
        }
    }
}

pub trait HuffmanTable<T> {
    fn symbols_with_bits(&self, bits: u32) -> u32;
    fn get_symbol(&self, bits: u32, index: u32) -> Result<T, &str>;
}

pub struct NaturalNumberHuffmanTable {
    alignment: u32
}

impl NaturalNumberHuffmanTable {
    pub fn create_with_alignment(alignment: u32) -> NaturalNumberHuffmanTable {
        NaturalNumberHuffmanTable {
            alignment
        }
    }
}

impl HuffmanTable<u32> for NaturalNumberHuffmanTable {
    fn symbols_with_bits(&self, bits: u32) -> u32 {
        if bits > 0 && bits % self.alignment == 0 {
            1 << ((bits / self.alignment) * (self.alignment - 1))
        }
        else {
            0
        }
    }

    fn get_symbol(&self, bits: u32, index: u32) -> Result<u32, &str> {
        if bits == 0 || bits % self.alignment != 0 {
            Err("Invalid symbol")
        }
        else {
            let mut base = 0u32;
            let mut exp = (bits - 1) / self.alignment;
            while exp > 0 {
                base += 1 << (exp * (self.alignment - 1));
                exp -= 1;
            }

            Ok(base + index)
        }
    }
}