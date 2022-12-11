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
    natural4_table: NaturalNumberHuffmanTable,
    natural8_table: NaturalNumberHuffmanTable
}

impl<'a> SdbReader<'a> {
    fn read(mut self) -> Result<huffman::DefinedHuffmanTable<char>, ReadError> {
        self.stream.read_symbol(&self.natural8_table).and_then(|symbol_array_count| {
            println!("Found {} symbol arrays", symbol_array_count);
            self.stream.read_table(&self.natural8_table, &self.natural4_table, InputBitStream::read_character, InputBitStream::read_diff_character)
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
