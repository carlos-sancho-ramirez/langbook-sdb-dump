use std::fs::File;
use std::io::Bytes;
use crate::file_utils;

pub struct InputBitStream<'a> {
    bytes: &'a mut Bytes<File>,
    buffer: u8,
    remaining: u32
}

impl<'a> InputBitStream<'a> {
    pub fn read_boolean(&mut self) -> Result<bool, file_utils::ReadError> {
        if self.remaining == 0 {
            self.buffer = file_utils::read_u8(self.bytes)?;
            self.remaining = 8;
        }

        let result = (self.buffer & 1) != 0;
        self.buffer >>= 1;
        self.remaining -= 1;
        Ok(result)
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