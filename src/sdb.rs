use std::collections::HashMap;
use std::fmt::{Display, Formatter, Write};
use std::hash::{Hash, Hasher};
use crate::file_utils::ReadError;
use crate::huffman::{HuffmanTable, InputBitStream, IntegerNumberHuffmanTable, NaturalNumberHuffmanTable, RangedIntegerHuffmanTable};

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

pub struct Language {
    code: LanguageCode,
    number_of_alphabets: usize
}

pub struct SymbolArrayIndex {
    index: usize
}

#[derive(Copy, Clone)]
pub struct Alphabet {
    index: usize
}

impl PartialEq<Self> for Alphabet {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Eq for Alphabet {
}

impl Hash for Alphabet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state)
    }
}

pub struct Conversion {
    source: Alphabet,
    target: Alphabet,
    pairs: Vec<(SymbolArrayIndex, SymbolArrayIndex)>
}

pub struct CorrelationIndex {
    index: usize
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CorrelationArrayIndex {
    index: usize
}

impl Hash for CorrelationArrayIndex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state)
    }
}

pub struct Acceptation {
    pub concept: u32,
    pub correlation_array_index: CorrelationArrayIndex
}

pub struct SdbReader<'a> {
    stream: InputBitStream<'a>,
    natural2_table: NaturalNumberHuffmanTable,
    natural3_table: NaturalNumberHuffmanTable,
    natural4_table: NaturalNumberHuffmanTable,
    natural8_table: NaturalNumberHuffmanTable,
    integer8_table: IntegerNumberHuffmanTable
}

pub struct SdbReadResult {
    pub symbol_arrays: Vec<String>,
    pub languages: Vec<Language>,
    pub conversions: Vec<Conversion>,
    pub max_concept: usize,
    pub correlations: Vec<HashMap<Alphabet, SymbolArrayIndex>>,
    pub correlation_arrays: Vec<Vec<CorrelationIndex>>,
    pub acceptations: Vec<Acceptation>
}

impl<'a> SdbReader<'a> {
    pub fn new(stream: InputBitStream<'a>) -> Self {
        Self {
            stream,
            natural2_table: NaturalNumberHuffmanTable::create_with_alignment(2),
            natural3_table: NaturalNumberHuffmanTable::create_with_alignment(3),
            natural4_table: NaturalNumberHuffmanTable::create_with_alignment(4),
            natural8_table: NaturalNumberHuffmanTable::create_with_alignment(8),
            integer8_table: IntegerNumberHuffmanTable::create_with_alignment(8)
        }
    }

    fn read_symbol_arrays(&mut self, symbol_array_count: usize, symbol_arrays_length_table: impl HuffmanTable<u32>, chars_table: impl HuffmanTable<char>) -> Result<Vec<String>, ReadError> {
        let mut symbol_arrays: Vec<String> = Vec::with_capacity(symbol_array_count);
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
        let mut languages: Vec<Language> = Vec::with_capacity(usize::try_from(language_count).unwrap());
        for _ in 0..language_count {
            let table = RangedIntegerHuffmanTable::new(first_valid_lang_code, last_valid_lang_code);
            let raw_lang_code = self.stream.read_symbol(&table)?;
            let code = LanguageCode::new(raw_lang_code);
            first_valid_lang_code = raw_lang_code + 1;

            let number_of_alphabets = usize::try_from(self.stream.read_symbol(&self.natural2_table)?).expect("Too many alphabets for a single language");

            languages.push(Language {
                code,
                number_of_alphabets
            })
        }

        Ok(languages)
    }

    fn read_conversions(&mut self, alphabet_count: usize, symbol_array_count: usize) -> Result<Vec<Conversion>, ReadError> {
        let number_of_conversions = self.stream.read_symbol(&self.natural8_table)?;
        let symbol_array_table = RangedIntegerHuffmanTable::new(0, u32::try_from(symbol_array_count - 1).unwrap());
        let max_valid_alphabet = alphabet_count - 1;
        let mut min_source_alphabet = 0;
        let mut min_target_alphabet = 0;
        let mut conversions: Vec<Conversion> = Vec::with_capacity(usize::try_from(number_of_conversions).unwrap());
        for _ in 0..number_of_conversions {
            let source_alphabet_table = RangedIntegerHuffmanTable::new(min_source_alphabet, u32::try_from(max_valid_alphabet).unwrap());
            let raw_source_alphabet = self.stream.read_symbol(&source_alphabet_table)?;
            let source_alphabet = Alphabet {
                index: usize::try_from(raw_source_alphabet).unwrap()
            };

            if min_source_alphabet != raw_source_alphabet {
                min_target_alphabet = 0;
                min_source_alphabet = raw_source_alphabet;
            }

            let target_alphabet_table = RangedIntegerHuffmanTable::new(min_target_alphabet, u32::try_from(max_valid_alphabet).unwrap());
            let raw_target_alphabet = self.stream.read_symbol(&target_alphabet_table)?;
            let target_alphabet = Alphabet {
                index: usize::try_from(raw_target_alphabet).unwrap()
            };

            min_target_alphabet = raw_target_alphabet + 1;

            let pair_count = self.stream.read_symbol(&self.natural8_table)?;
            let mut pairs: Vec<(SymbolArrayIndex, SymbolArrayIndex)> = Vec::with_capacity(usize::try_from(pair_count).unwrap());
            for _ in 0..pair_count {
                let source = SymbolArrayIndex {
                    index: usize::try_from(self.stream.read_symbol(&symbol_array_table)?).unwrap()
                };

                let target = SymbolArrayIndex {
                    index: usize::try_from(self.stream.read_symbol(&symbol_array_table)?).unwrap()
                };
                pairs.push((source, target));
            }

            conversions.push(Conversion {
                source: source_alphabet,
                target: target_alphabet,
                pairs
            })
        }

        Ok(conversions)
    }

    fn read_correlations(&mut self, alphabet_count: usize, symbol_array_count: usize) -> Result<Vec<HashMap<Alphabet, SymbolArrayIndex>>, ReadError> {
        let number_of_correlations = self.stream.read_symbol(&self.natural8_table)?;
        let alphabet_count_u32 = u32::try_from(alphabet_count).unwrap();
        let symbol_array_count_u32 = u32::try_from(symbol_array_count).unwrap();
        let mut correlations: Vec<HashMap<Alphabet, SymbolArrayIndex>> = Vec::with_capacity(usize::try_from(number_of_correlations).unwrap());
        if number_of_correlations > 0 {
            // The serialization of correlations can be improved in several ways:
            // - There can be only one correlation with length 0. It could be serialised with a single bit: 0 (not present), 1 (present at the beginning)
            // - If correlations cannot mix alphabets from different languages, then we could reduce the number of possible keys once we know the first key, or even the language. For languages where only one alphabet is available, then the length and the key gets irrelevant
            // TODO: Improve codification for this table, it include lot of edge cases that should not be possible
            let length_table = self.stream.read_table(&self.integer8_table, &self.natural8_table, InputBitStream::read_symbol,InputBitStream::read_diff_i32)?;
            for _ in 0..number_of_correlations {
                let map_length = u32::try_from(self.stream.read_symbol(&length_table)?).unwrap();
                if map_length >= alphabet_count_u32 {
                    panic!("Map for correlation cannot be longer than the actual number of valid alphabets");
                }

                let mut map: HashMap<Alphabet, SymbolArrayIndex> = HashMap::with_capacity(usize::try_from(map_length).unwrap());
                if map_length > 0 {
                    let key_table = RangedIntegerHuffmanTable::new(0, alphabet_count_u32 - map_length);
                    let value_table = RangedIntegerHuffmanTable::new(0, symbol_array_count_u32 - 1);
                    let mut raw_key = self.stream.read_symbol(&key_table)?;
                    let key = Alphabet {
                        index: usize::try_from(raw_key).unwrap()
                    };

                    let value = SymbolArrayIndex {
                        index: usize::try_from(self.stream.read_symbol(&value_table)?).unwrap()
                    };
                    map.insert(key, value);
                    for map_index in 1..map_length {
                        let key_diff_table = RangedIntegerHuffmanTable::new(raw_key + 1, alphabet_count_u32 - map_length + map_index);
                        raw_key = self.stream.read_symbol(&key_diff_table)?;
                        let key = Alphabet {
                            index: usize::try_from(raw_key).unwrap()
                        };

                        let value = SymbolArrayIndex {
                            index: usize::try_from(self.stream.read_symbol(&value_table)?).unwrap()
                        };

                        map.insert(key, value);
                    }
                }
                correlations.push(map);
            }
        }

        Ok(correlations)
    }

    fn read_correlation_arrays(&mut self, number_of_correlations: usize) -> Result<Vec<Vec<CorrelationIndex>>, ReadError> {
        let number_of_arrays = self.stream.read_symbol(&self.natural8_table)?;
        let mut arrays: Vec<Vec<CorrelationIndex>> = Vec::with_capacity(usize::try_from(number_of_arrays).unwrap());
        if number_of_arrays > 0 {
            let correlation_table = RangedIntegerHuffmanTable::new(0, u32::try_from(number_of_correlations).unwrap() - 1);
            // TODO: Improve codification for this table, it include lot of edge cases that should not be possible
            let length_table = self.stream.read_table(&self.integer8_table, &self.natural8_table, InputBitStream::read_symbol,InputBitStream::read_diff_i32)?;

            for _ in 0..number_of_arrays {
                let array_length = usize::try_from(self.stream.read_symbol(&length_table)?).unwrap();
                let mut array: Vec<CorrelationIndex> = Vec::with_capacity(array_length);
                for _ in 0..array_length {
                    array.push(CorrelationIndex {
                        index: usize::try_from(self.stream.read_symbol(&correlation_table)?).unwrap()
                    });
                }
                arrays.push(array);
            }
        }

        Ok(arrays)
    }

    fn read_acceptations(&mut self, min_valid_concept: usize, max_valid_concept: usize, correlation_array_count: usize) -> Result<Vec<Acceptation>, ReadError> {
        let number_of_entries = self.stream.read_symbol(&self.natural8_table)?;
        let mut result: Vec<Acceptation> = Vec::new();
        if number_of_entries > 0 {
            let min_valid_concept_u32 = u32::try_from(min_valid_concept).unwrap();
            let max_valid_concept_u32 = u32::try_from(max_valid_concept).unwrap();
            let correlation_array_count_u32 = u32::try_from(correlation_array_count).unwrap();

            // TODO: Improve codification for this table, it include some edge cases that should not be possible, like negative values for lengths
            let correlation_array_set_length_table = self.stream.read_table(&self.integer8_table, &self.natural8_table, InputBitStream::read_symbol, InputBitStream::read_diff_i32)?;
            let concept_table = RangedIntegerHuffmanTable::new(min_valid_concept_u32, max_valid_concept_u32);
            for _ in 0..number_of_entries {
                let concept = self.stream.read_symbol(&concept_table)?;
                let length = self.stream.read_symbol(&correlation_array_set_length_table)?;
                if length <= 0 {
                    panic!("Unexpected length {}", length);
                }

                let length_u32 = u32::try_from(length).unwrap();
                let symbol_table = RangedIntegerHuffmanTable::new(0, correlation_array_count_u32 - length_u32);
                let mut value = self.stream.read_symbol(&symbol_table)?;
                result.push(Acceptation {
                    concept,
                    correlation_array_index: CorrelationArrayIndex {
                        index: usize::try_from(value).unwrap()
                    }
                });

                for set_entry_index in 1..length_u32 {
                    let symbol_diff_table = RangedIntegerHuffmanTable::new(value + 1, correlation_array_count_u32 - length_u32 + set_entry_index);
                    value += self.stream.read_symbol(&symbol_diff_table)? + 1;
                    result.push(Acceptation {
                        concept,
                        correlation_array_index: CorrelationArrayIndex {
                            index: usize::try_from(value).unwrap()
                        }
                    });
                }
            }
        }

        Ok(result)
    }

    pub fn read(mut self) -> Result<SdbReadResult, ReadError> {
        let symbol_array_count = usize::try_from(self.stream.read_symbol(&self.natural8_table)?).unwrap();
        let chars_table = self.stream.read_table(&self.natural8_table, &self.natural4_table, InputBitStream::read_character, InputBitStream::read_diff_character)?;
        let symbol_arrays_length_table = self.stream.read_table(&self.natural8_table, &self.natural3_table, InputBitStream::read_symbol, InputBitStream::read_diff_u32)?;
        let symbol_arrays = self.read_symbol_arrays(symbol_array_count, symbol_arrays_length_table, chars_table)?;
        let languages = self.read_languages()?;

        if symbol_array_count == 0 {
            todo!("Implementation missing when symbol array count is 0");
        }

        let mut alphabet_count: usize = 0;
        for language in &languages {
            alphabet_count += language.number_of_alphabets;
        }

        let conversions = self.read_conversions(alphabet_count, symbol_array_count)?;
        let max_concept_u32 = self.stream.read_symbol(&self.natural8_table)?;
        let max_concept = usize::try_from(max_concept_u32).unwrap();
        let correlations = self.read_correlations(alphabet_count, symbol_array_count)?;
        let correlation_arrays = self.read_correlation_arrays(correlations.len())?;
        let acceptations = self.read_acceptations(1, max_concept, correlation_arrays.len())?;

        Ok(SdbReadResult {
            symbol_arrays,
            languages,
            conversions,
            max_concept,
            correlations,
            correlation_arrays,
            acceptations
        })
    }
}

impl SdbReadResult {
    pub fn get_complete_correlation(&self, correlation_array_index: CorrelationArrayIndex) -> HashMap<Alphabet, String> {
        let mut result: HashMap<Alphabet, String> = HashMap::new();
        let array: &Vec<CorrelationIndex> = &self.correlation_arrays[correlation_array_index.index];
        let array_length = array.len();
        if array_length == 0 {
            return result;
        }

        let correlation: &HashMap<Alphabet, SymbolArrayIndex> = &self.correlations[array[0].index];
        for (key, value) in correlation {
            result.insert(*key, self.symbol_arrays[value.index].clone());
        }

        if array_length > 1 {
            for array_index in 1..array_length {
                for (key, value) in self.correlations[array[array_index].index].iter() {
                    let text = &self.symbol_arrays[value.index];
                    result.get_mut(key).unwrap().push_str(text);
                }
            }
        }

        result
    }
}
