use std::env;
use std::fs::File;
use std::io::Read;
use huffman::InputBitStream;
use huffman::NaturalNumberHuffmanTable;
use crate::file_utils::ReadError;

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
    symbol_arrays: Vec<String>
}

impl<'a> SdbReader<'a> {
    fn read(mut self) -> Result<SdbReadResult, ReadError> {
        let symbol_array_count = self.stream.read_symbol(&self.natural8_table)?;
        let chars_table = self.stream.read_table(&self.natural8_table, &self.natural4_table, InputBitStream::read_character, InputBitStream::read_diff_character)?;
        let symbol_arrays_length_table = self.stream.read_table(&self.natural8_table, &self.natural3_table, InputBitStream::read_symbol, InputBitStream::read_diff_u32)?;

        let mut symbol_arrays: Vec<String> = Vec::new();
        for _ in 0..symbol_array_count {
            let length = self.stream.read_symbol(&symbol_arrays_length_table)?;
            let mut array = String::new();
            for _ in 0..length {
                array.push(self.stream.read_symbol(&chars_table)?);
            }
            symbol_arrays.push(array);
        }

        Ok(SdbReadResult {
            symbol_arrays
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
                        Ok(result) => {
                            let entry_count = result.symbol_arrays.len();
                            println!("Symbol arrays read - {} entries:", entry_count);
                            for str in result.symbol_arrays {
                                println!("  {}", str);
                            }
                        },
                        Err(err) => println!("Error found: {}", err.message)
                    }
                }
            }
        }
    }
}
