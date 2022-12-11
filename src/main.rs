use std::env;
use std::fs::File;
use std::io::Read;
use huffman::InputBitStream;
use huffman::NaturalNumberHuffmanTable;
use crate::file_utils::ReadError;
use crate::huffman::DefinedHuffmanTable;

pub mod file_utils;
pub mod huffman;

struct Params {
    input_file_name: String
}

fn obtain_arguments() -> Result<Params, String> {
    let mut next_is_input = false;
    let mut input_file_name: Option<String> = None;
    let mut is_first = true;
    for arg in env::args() {
        if is_first {
            is_first = false;
        }
        else if next_is_input {
            next_is_input = false;
            input_file_name = Some(arg);
        }
        else if arg == "-i" {
            if input_file_name.is_none() {
                next_is_input = true
            }
            else {
                return Err(String::from("Input file already set"));
            }
        }
        else {
            let mut s = String::from("Invalid argument ");
            s.push_str(&arg);
            return Err(s);
        }
    }

    match input_file_name {
        Some(name) => Ok(Params {
            input_file_name: name
        }),
        None => {
            let mut s = String::from("Missing input file: try ");
            s.push_str(&env::args().next().expect("wtf?"));
            s.push_str(" -i <sdb-file>");
            Err(s)
        }
    }
}

struct SdbReader<'a> {
    stream: InputBitStream<'a>,
    natural3_table: NaturalNumberHuffmanTable,
    natural4_table: NaturalNumberHuffmanTable,
    natural8_table: NaturalNumberHuffmanTable
}

struct SdbReadResult {
    symbol_array_count: u32,
    chars_table: DefinedHuffmanTable<char>,
    symbol_arrays_length_table: DefinedHuffmanTable<u32>
}

impl<'a> SdbReader<'a> {
    fn read(mut self) -> Result<SdbReadResult, ReadError> {
        let symbol_array_count = self.stream.read_symbol(&self.natural8_table)?;
        println!("Found {} symbol arrays", symbol_array_count);
        println!("Characters table:");
        let chars_table = self.stream.read_table(&self.natural8_table, &self.natural4_table, InputBitStream::read_character, InputBitStream::read_diff_character)?;
        println!();
        println!("Symbol arrays length table:");
        let symbol_arrays_length_table = self.stream.read_table(&self.natural8_table, &self.natural3_table, InputBitStream::read_symbol, InputBitStream::read_diff_u32)?;
        Ok(SdbReadResult {
            symbol_array_count,
            chars_table,
            symbol_arrays_length_table
        })
    }
}

fn main() {
    match obtain_arguments() {
        Err(text) => println!("{}", text),
        Ok(params) => {
            println!("Reading file {}", params.input_file_name);
            match File::open(&params.input_file_name) {
                Err(_) => println!("Unable to open file {}", params.input_file_name),
                Ok(file) => {
                    let mut bytes = file.bytes();
                    match file_utils::assert_next_is_same_text(&mut bytes, "SDB\x01").and_then(|_| {
                        let reader = SdbReader {
                            stream: InputBitStream::from(&mut bytes),
                            natural3_table: NaturalNumberHuffmanTable::create_with_alignment(3),
                            natural4_table: NaturalNumberHuffmanTable::create_with_alignment(4),
                            natural8_table: NaturalNumberHuffmanTable::create_with_alignment(8)
                        };
                        reader.read()
                    }) {
                        Ok(_) => println!("Table read"),
                        Err(err) => println!("Error found: {}", err.message)
                    }
                }
            }
        }
    }
}
