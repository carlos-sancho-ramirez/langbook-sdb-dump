use std::env;
use std::fs::File;
use std::io::Read;
use huffman::InputBitStream;

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

fn main() {
    match obtain_arguments() {
        Err(text) => println!("{}", text),
        Ok(params) => {
            println!("Reading file {}", params.input_file_name);
            match File::open(&params.input_file_name) {
                Err(_) => println!("Unable to open file {}", params.input_file_name),
                Ok(file) => {
                    let mut bytes = file.bytes();
                    match file_utils::assert_next_is_same_text(&mut bytes, "SDB\x01") {
                        Ok(_) => {
                            println!("Displaying first bits:");
                            let mut stream = InputBitStream::from(&mut bytes);
                            for _ in 0..20 {
                                println!("  {}", stream.read_boolean().expect("Unable to read boolean"))
                            }
                        },
                        Err(err) => println!("Error found: {}", err.message)
                    }
                }
            }
        }
    }
}
