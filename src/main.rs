use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::Read;
use huffman::InputBitStream;
use crate::sdb::{CorrelationArrayIndex, SdbReader, SdbReadResult};

pub mod file_utils;
pub mod huffman;
pub mod sdb;

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
                    match file_utils::assert_next_is_same_text(&mut bytes, "SDB\x01").and_then(|_| {
                        SdbReader::new(InputBitStream::from(&mut bytes)).read()
                    }) {
                        Ok(result) => {
                            println!("Symbol arrays read - {} entries", result.symbol_arrays.len());
                            println!("Languages read - {} languages found" , result.languages.len());
                            println!("Conversions read - {} conversions found" , result.conversions.len());
                            println!("Found {} concepts", result.max_concept);
                            println!("Correlations read - {} correlations found", result.correlations.len());
                            println!("Correlation arrays read - {} correlation arrays found", result.correlation_arrays.len());
                            println!("Acceptations read - {} acceptations found", result.acceptations.len());
                            println!("Definitions read - {} definitions found", result.definitions.len());

                            fn concept_to_string(result: &SdbReadResult, concept: usize) -> String {
                                for acc in result.acceptations.iter() {
                                    if acc.concept == concept {
                                        return result.get_complete_correlation(acc.correlation_array_index).into_values().reduce(|a, b| {
                                            let mut c = String::new();
                                            c.push_str(&a);
                                            c.push('/');
                                            c.push_str(&b);
                                            c
                                        }).unwrap()
                                    }
                                }

                                panic!("No suitable string found for concept {}", concept);
                            }

                            for (concept, definition) in result.definitions.iter() {
                                let mut text = String::new();
                                text.push_str(&concept_to_string(&result, *concept));
                                text.push_str(": ");
                                text.push_str(&concept_to_string(&result, definition.base_concept));
                                for complement in definition.complements.iter() {
                                    text.push_str(" + ");
                                    text.push_str(&concept_to_string(&result, *complement));
                                }

                                println!("  {}", text);
                            }
                        },
                        Err(err) => println!("Error found: {}", err.message)
                    }
                }
            }
        }
    }
}
