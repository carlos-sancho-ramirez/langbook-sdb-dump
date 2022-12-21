use std::env;
use std::fmt::{Display, Formatter, Write};
use std::fs::File;
use std::io::Read;
use std::ops::Range;
use huffman::InputBitStream;
use huffman::NaturalNumberHuffmanTable;
use crate::file_utils::ReadError;
use crate::huffman::{HuffmanTable, RangedIntegerHuffmanTable};

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

struct LanguageCode {
    code: u16
}

impl LanguageCode {
    fn new(code: u32) -> Self {
        if code >= 26 * 26 {
            panic!("Invalid language code");
        }

        Self {
            code: u16::try_from(code).expect("Invalid language code")
        }
    }
}

impl Display for LanguageCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char(char::try_from(u32::from(self.code) / 26 + ('a' as u32)).expect(""))?;
        f.write_char(char::try_from(u32::from(self.code) % 26 + ('a' as u32)).expect(""))
    }
}

struct Language {
    code: LanguageCode,
    number_of_alphabets: u8
}

struct Conversion {
    source_alphabet: u32,
    target_alphabet: u32,
    sources: Vec<u32>,
    targets: Vec<u32>
}

struct SdbReader<'a> {
    stream: InputBitStream<'a>,
    natural2_table: NaturalNumberHuffmanTable,
    natural3_table: NaturalNumberHuffmanTable,
    natural4_table: NaturalNumberHuffmanTable,
    natural8_table: NaturalNumberHuffmanTable
}

struct SdbReadResult {
    symbol_arrays: Vec<String>,
    languages: Vec<Language>,
    conversions: Vec<Conversion>
}

impl<'a> SdbReader<'a> {
    fn read_symbol_arrays(&mut self, symbol_array_count: u32, symbol_arrays_length_table: impl HuffmanTable<u32>, chars_table: impl HuffmanTable<char>) -> Result<Vec<String>, ReadError> {
        let mut symbol_arrays: Vec<String> = Vec::new();
        for _ in 0..symbol_array_count {
            let length = self.stream.read_symbol(&symbol_arrays_length_table)?;
            let mut array = String::new();
            for _ in 0..length {
                array.push(self.stream.read_symbol(&chars_table)?);
            }
            symbol_arrays.push(array);
        }

        Ok(symbol_arrays)
    }

    fn read_languages(&mut self) -> Result<Vec<Language>, ReadError> {
        let language_count = self.stream.read_symbol(&self.natural8_table)?;

        let last_valid_lang_code = 26 * 26 - 1;
        let mut first_valid_lang_code = 0;
        let mut languages: Vec<Language> = Vec::new();
        for _ in 0..language_count {
            let table = RangedIntegerHuffmanTable::new(first_valid_lang_code, last_valid_lang_code);
            let raw_lang_code = self.stream.read_symbol(&table)?;
            let code = LanguageCode::new(raw_lang_code);
            first_valid_lang_code = raw_lang_code + 1;

            let number_of_alphabets = match u8::try_from(self.stream.read_symbol(&self.natural2_table)?) {
                Ok(x) => x,
                Err(_) => return Err(ReadError::from("Too many alphabets for a single language"))
            };

            languages.push(Language {
                code,
                number_of_alphabets
            })
        }

        Ok(languages)
    }

    fn read_conversions(&mut self, valid_alphabets: &Range<u32>, valid_symbol_arrays: &Range<u32>) -> Result<Vec<Conversion>, ReadError> {
        let number_of_conversions = self.stream.read_symbol(&self.natural8_table)?;
        let symbol_array_table = RangedIntegerHuffmanTable::from(valid_symbol_arrays);
        let max_valid_alphabet = valid_alphabets.end - 1;
        let mut min_source_alphabet = valid_alphabets.start;
        let mut min_target_alphabet = valid_alphabets.start;
        let mut conversions: Vec<Conversion> = Vec::new();
        for _ in 0..number_of_conversions {
            let source_alphabet_table = RangedIntegerHuffmanTable::new(min_source_alphabet, max_valid_alphabet);
            let source_alphabet = self.stream.read_symbol(&source_alphabet_table)?;

            if min_source_alphabet != source_alphabet {
                min_target_alphabet = valid_alphabets.start;
                min_source_alphabet = source_alphabet;
            }

            let target_alphabet_table = RangedIntegerHuffmanTable::new(min_target_alphabet, max_valid_alphabet);
            let target_alphabet = self.stream.read_symbol(&target_alphabet_table)?;
            min_target_alphabet = target_alphabet + 1;

            let pair_count = self.stream.read_symbol(&self.natural8_table)?;
            let mut sources: Vec<u32> = Vec::new();
            let mut targets: Vec<u32> = Vec::new();
            for _ in 0..pair_count {
                sources.push(self.stream.read_symbol(&symbol_array_table)?);
                targets.push(self.stream.read_symbol(&symbol_array_table)?);
            }

            conversions.push(Conversion {
                source_alphabet,
                target_alphabet,
                sources,
                targets
            })
        }

        Ok(conversions)
    }

    fn read(mut self) -> Result<SdbReadResult, ReadError> {
        let symbol_array_count = self.stream.read_symbol(&self.natural8_table)?;
        let chars_table = self.stream.read_table(&self.natural8_table, &self.natural4_table, InputBitStream::read_character, InputBitStream::read_diff_character)?;
        let symbol_arrays_length_table = self.stream.read_table(&self.natural8_table, &self.natural3_table, InputBitStream::read_symbol, InputBitStream::read_diff_u32)?;
        let symbol_arrays = self.read_symbol_arrays(symbol_array_count, symbol_arrays_length_table, chars_table)?;
        let languages = self.read_languages()?;

        if symbol_array_count == 0 {
            todo!("Implementation missing when symbol array count is 0");
        }

        let mut alphabet_count: u32 = 0;
        for language in &languages {
            alphabet_count += u32::from(language.number_of_alphabets);
        }
        let valid_alphabets = 0..alphabet_count;
        let valid_symbol_arrays = 0..symbol_array_count;

        let conversions = self.read_conversions(&valid_alphabets, &valid_symbol_arrays)?;

        Ok(SdbReadResult {
            symbol_arrays,
            languages,
            conversions
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
                            natural2_table: NaturalNumberHuffmanTable::create_with_alignment(2),
                            natural3_table: NaturalNumberHuffmanTable::create_with_alignment(3),
                            natural4_table: NaturalNumberHuffmanTable::create_with_alignment(4),
                            natural8_table: NaturalNumberHuffmanTable::create_with_alignment(8)
                        };
                        reader.read()
                    }) {
                        Ok(result) => {
                            println!("Symbol arrays read - {} entries", result.symbol_arrays.len());
                            println!("Languages read - {} languages found" , result.languages.len());
                            println!("Conversions read - {} conversions found" , result.conversions.len());
                            for conversion_index in 0..result.conversions.len() {
                                println!("  Conversion #{}", conversion_index);
                                for pair_index in 0..result.conversions[conversion_index].sources.len() {
                                    let source_symbol_array_index = result.conversions[conversion_index].sources[pair_index] as usize;
                                    let target_symbol_array_index = result.conversions[conversion_index].targets[pair_index] as usize;
                                    println!("    {} -> {}", result.symbol_arrays[source_symbol_array_index], result.symbol_arrays[target_symbol_array_index]);
                                }
                            }
                        },
                        Err(err) => println!("Error found: {}", err.message)
                    }
                }
            }
        }
    }
}
