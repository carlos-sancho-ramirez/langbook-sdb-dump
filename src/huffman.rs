use std::fmt::Display;
use std::fs::File;
use std::io::Bytes;
use std::ops::Range;
use crate::file_utils;
use file_utils::ReadError;

pub struct InputBitStream<'a> {
    bytes: &'a mut Bytes<File>,
    buffer: u8,
    remaining: u32
}

impl<'a> InputBitStream<'a> {
    pub fn read_boolean(&mut self) -> Result<bool, ReadError> {
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

    pub fn read_diff_u32<T: HuffmanTable<u32>>(&mut self, table: &T, previous: u32) -> Result<u32, ReadError> {
        Ok(previous + self.read_symbol(table)? + 1u32)
    }

    pub fn read_diff_i32<T: HuffmanTable<u32>>(&mut self, table: &T, previous: i32) -> Result<i32, ReadError> {
        match i32::try_from(self.read_symbol(table)?) {
            Ok(x) => Ok(previous + x + 1),
            Err(_) => panic!("Out of range")
        }
    }

    pub fn read_character<T: HuffmanTable<u32>>(&mut self, table: &T) -> Result<char, ReadError> {
        match char::from_u32(self.read_symbol(table)?) {
            Some(ch) => Ok(ch),
            None => Err(ReadError::from("Unable to convert char"))
        }
    }

    pub fn read_diff_character<T: HuffmanTable<u32>>(&mut self, table: &T, previous: char) -> Result<char, ReadError> {
        match char::from_u32(self.read_symbol(table)? + (previous as u32) + 1) {
            Some(ch) => Ok(ch),
            None => Err(ReadError::from("Unable to convert char"))
        }
    }

    pub fn read_table<S : Copy + Display, T1, T2>(&mut self, table1: &T1, table2: &T2, supplier: impl Fn(&mut Self, &T1) -> Result<S, ReadError>, diff_supplier: impl Fn(&mut Self, &T2, S) -> Result<S, ReadError>) -> Result<DefinedHuffmanTable<S>, ReadError> {
        let mut level_lengths: Vec<u32> = Vec::new();
        let mut max = 1;
        while max > 0 {
            let ranged_integer_huffman_table = RangedIntegerHuffmanTable::new(0, max);
            let level_length = self.read_symbol(&ranged_integer_huffman_table)?;
            level_lengths.push(level_length);
            max -= level_length;
            max <<= 1;
        }

        let mut level_indexes: Vec<usize> = Vec::new();
        let mut symbols: Vec<S> = Vec::new();

        for index in 0..level_lengths.len() {
            if index > 0 {
                level_indexes.push(symbols.len());
            }

            let level_length = level_lengths[index];
            if level_length > 0 {
                let mut element = supplier(self, &table1)?;
                symbols.push(element);

                for _ in 1..level_length {
                    element = diff_supplier(self, &table2, element)?;
                    symbols.push(element);
                }
            }
        }

        Ok(DefinedHuffmanTable {
            level_indexes,
            symbols
        })
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
    pub fn create_with_alignment(alignment: u32) -> Self {
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

pub struct NaturalUsizeHuffmanTable {
    alignment: u32
}

impl NaturalUsizeHuffmanTable {
    pub fn create_with_alignment(alignment: u32) -> Self {
        NaturalUsizeHuffmanTable {
            alignment
        }
    }
}

impl HuffmanTable<usize> for NaturalUsizeHuffmanTable {
    fn symbols_with_bits(&self, bits: u32) -> u32 {
        if bits > 0 && bits % self.alignment == 0 {
            1 << ((bits / self.alignment) * (self.alignment - 1))
        }
        else {
            0
        }
    }

    fn get_symbol(&self, bits: u32, index: u32) -> Result<usize, &str> {
        if bits == 0 || bits % self.alignment != 0 {
            Err("Invalid symbol")
        }
        else {
            let mut base = 0usize;
            let mut exp = (bits - 1) / self.alignment;
            while exp > 0 {
                base += 1 << (exp * (self.alignment - 1));
                exp -= 1;
            }

            Ok(base + usize::try_from(index).unwrap())
        }
    }
}

pub struct IntegerNumberHuffmanTable {
    alignment: u32
}

impl IntegerNumberHuffmanTable {
    pub fn create_with_alignment(alignment: u32) -> Self {
        IntegerNumberHuffmanTable {
            alignment
        }
    }
}

impl HuffmanTable<i32> for IntegerNumberHuffmanTable {
    fn symbols_with_bits(&self, bits: u32) -> u32 {
        if bits > 0 && bits % self.alignment == 0 {
            1 << ((bits / self.alignment) * (self.alignment - 1))
        }
        else {
            0
        }
    }

    fn get_symbol(&self, bits: u32, index: u32) -> Result<i32, &str> {
        if bits == 0 || bits % self.alignment != 0 {
            Err("Invalid symbol")
        }
        else {
            let symbols_per_segment = self.symbols_with_bits(bits) / 2;
            let segment_alignment = self.alignment - 1;
            let mut base = 0i32;

            Ok(if index < symbols_per_segment {
                let multiplier = (bits - 1) / self.alignment;
                if multiplier > 0 {
                    let mut exp = multiplier * segment_alignment - 1;
                    while exp > 0 {
                        base += 1 << exp;
                        exp -= segment_alignment;
                    }
                }

                base + i32::try_from(index).unwrap()
            }
            else {
                let mut base = 0i32;
                let mut exp = (bits / self.alignment) * segment_alignment - 1;
                while exp > 0 {
                    base -= 1 << exp;
                    exp -= segment_alignment;
                }

                base + i32::try_from(index - symbols_per_segment).unwrap()
            })
        }
    }
}

pub struct RangedIntegerHuffmanTable {
    min: u32,
    max: u32,
    max_bits: u32,
    limit: u32
}

impl RangedIntegerHuffmanTable {
    pub fn new(min: u32, max: u32) -> Self {
        if max < min {
            panic!("Invalid range");
        }

        let possibilities = max - min + 1;
        let mut max_bits = 0;
        while possibilities > (1 << max_bits) {
            max_bits += 1;
        }

        let limit = (1 << max_bits) - possibilities;

        Self {
            min,
            max,
            max_bits,
            limit
        }
    }
}

impl From<&Range<u32>> for RangedIntegerHuffmanTable {
    fn from(range: &Range<u32>) -> Self {
        RangedIntegerHuffmanTable::new(range.start, range.end - 1)
    }
}

impl HuffmanTable<u32> for RangedIntegerHuffmanTable {
    fn symbols_with_bits(&self, bits: u32) -> u32 {
        if bits == self.max_bits {
            self.max - self.min + 1 - self.limit
        }
        else if bits == self.max_bits - 1 {
            self.limit
        }
        else {
            0
        }
    }

    fn get_symbol(&self, bits: u32, index: u32) -> Result<u32, &str> {
        if bits == self.max_bits {
            Ok(index + self.limit + self.min)
        }
        else if bits == self.max_bits - 1 {
            Ok(index + self.min)
        }
        else {
            Err("Invalid number of bits")
        }
    }
}

pub struct RangedNaturalUsizeHuffmanTable {
    min: usize,
    max: usize,
    max_bits: u32,
    limit: u32
}

impl RangedNaturalUsizeHuffmanTable {
    pub fn new(min: usize, max: usize) -> Self {
        if max < min {
            panic!("Invalid range");
        }

        let possibilities = u32::try_from(max - min + 1).unwrap();
        let mut max_bits = 0;
        while possibilities > (1 << max_bits) {
            max_bits += 1;
        }

        let limit = (1 << max_bits) - possibilities;

        Self {
            min,
            max,
            max_bits,
            limit
        }
    }
}

impl From<&Range<usize>> for RangedNaturalUsizeHuffmanTable {
    fn from(range: &Range<usize>) -> Self {
        RangedNaturalUsizeHuffmanTable::new(range.start, range.end - 1)
    }
}

impl HuffmanTable<usize> for RangedNaturalUsizeHuffmanTable {
    fn symbols_with_bits(&self, bits: u32) -> u32 {
        if bits == self.max_bits {
            u32::try_from(self.max - self.min).unwrap() + 1 - self.limit
        }
        else if bits == self.max_bits - 1 {
            self.limit
        }
        else {
            0
        }
    }

    fn get_symbol(&self, bits: u32, index: u32) -> Result<usize, &str> {
        if bits == self.max_bits {
            Ok(usize::try_from(index + self.limit).unwrap() + self.min)
        }
        else if bits == self.max_bits - 1 {
            Ok(usize::try_from(index).unwrap() + self.min)
        }
        else {
            Err("Invalid number of bits")
        }
    }
}

pub struct DefinedHuffmanTable<S> {
    level_indexes: Vec<usize>,
    symbols: Vec<S>
}

impl<S: Copy> HuffmanTable<S> for DefinedHuffmanTable<S> {
    fn symbols_with_bits(&self, bits: u32) -> u32 {
        let level_index = if bits == 0 {
            0
        }
        else {
            self.level_indexes[(bits - 1) as usize]
        };

        let next_level_index = if self.level_indexes.len() == (bits as usize) {
            self.symbols.len()
        }
        else {
            self.level_indexes[bits as usize]
        };

        (next_level_index - level_index) as u32
    }

    fn get_symbol(&self, bits: u32, index: u32) -> Result<S, &str> {
        let offset = if bits == 0 {
            0
        }
        else {
            self.level_indexes[(bits - 1) as usize]
        };

        Ok(self.symbols[offset + (index as usize)])
    }
}