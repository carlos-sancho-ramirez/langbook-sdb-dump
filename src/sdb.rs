use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter, Write};
use std::hash::{Hash, Hasher};
use crate::file_utils::ReadError;
use crate::huffman::{HuffmanTable, InputBitStream, IntegerNumberHuffmanTable, NaturalNumberHuffmanTable, NaturalUsizeHuffmanTable, RangedIntegerHuffmanTable, RangedNaturalUsizeHuffmanTable};

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
    pub concept: usize,
    pub correlation_array_index: CorrelationArrayIndex
}

pub struct Definition {
    pub base_concept: usize,
    pub complements: HashSet<usize>
}

pub struct SdbReader<'a> {
    stream: InputBitStream<'a>,
    natural3_table: NaturalNumberHuffmanTable,
    natural4_table: NaturalNumberHuffmanTable,
    natural8_table: NaturalNumberHuffmanTable,
    integer8_table: IntegerNumberHuffmanTable,
    natural2_usize_table: NaturalUsizeHuffmanTable,
    natural8_usize_table: NaturalUsizeHuffmanTable
}

pub struct SdbReadResult {
    pub symbol_arrays: Vec<String>,
    pub languages: Vec<Language>,
    pub conversions: Vec<Conversion>,
    pub max_concept: usize,
    pub correlations: Vec<HashMap<Alphabet, SymbolArrayIndex>>,
    pub correlation_arrays: Vec<Vec<CorrelationIndex>>,
    pub acceptations: Vec<Acceptation>,
    pub definitions: HashMap<usize, Definition>
}

impl<'a> SdbReader<'a> {
    pub fn new(stream: InputBitStream<'a>) -> Self {
        Self {
            stream,
            natural3_table: NaturalNumberHuffmanTable::create_with_alignment(3),
            natural4_table: NaturalNumberHuffmanTable::create_with_alignment(4),
            natural8_table: NaturalNumberHuffmanTable::create_with_alignment(8),
            integer8_table: IntegerNumberHuffmanTable::create_with_alignment(8),
            natural2_usize_table: NaturalUsizeHuffmanTable::create_with_alignment(2),
            natural8_usize_table: NaturalUsizeHuffmanTable::create_with_alignment(8)
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
        let language_count = self.stream.read_symbol(&self.natural8_usize_table)?;

        let last_valid_lang_code = 26 * 26 - 1;
        let mut first_valid_lang_code = 0;
        let mut languages: Vec<Language> = Vec::with_capacity(language_count);
        for _ in 0..language_count {
            let table = RangedIntegerHuffmanTable::new(first_valid_lang_code, last_valid_lang_code);
            let raw_lang_code = self.stream.read_symbol(&table)?;
            let code = LanguageCode::new(raw_lang_code);
            first_valid_lang_code = raw_lang_code + 1;

            let number_of_alphabets = self.stream.read_symbol(&self.natural2_usize_table)?;
            languages.push(Language {
                code,
                number_of_alphabets
            })
        }

        Ok(languages)
    }

    fn read_conversions(&mut self, alphabet_count: usize, symbol_array_count: usize) -> Result<Vec<Conversion>, ReadError> {
        let number_of_conversions = self.stream.read_symbol(&self.natural8_usize_table)?;
        let symbol_array_table = RangedIntegerHuffmanTable::new(0, u32::try_from(symbol_array_count - 1).unwrap());
        let max_valid_alphabet = alphabet_count - 1;
        let mut min_source_alphabet = 0usize;
        let mut min_target_alphabet = 0usize;
        let mut conversions: Vec<Conversion> = Vec::with_capacity(number_of_conversions);
        for _ in 0..number_of_conversions {
            let source_alphabet_table = RangedNaturalUsizeHuffmanTable::new(min_source_alphabet, max_valid_alphabet);
            let source_alphabet_index = self.stream.read_symbol(&source_alphabet_table)?;
            let source_alphabet = Alphabet {
                index: source_alphabet_index
            };

            if min_source_alphabet != source_alphabet_index {
                min_target_alphabet = 0usize;
                min_source_alphabet = source_alphabet_index;
            }

            let target_alphabet_table = RangedNaturalUsizeHuffmanTable::new(min_target_alphabet, max_valid_alphabet);
            let target_alphabet_index = self.stream.read_symbol(&target_alphabet_table)?;
            let target_alphabet = Alphabet {
                index: target_alphabet_index
            };

            min_target_alphabet = target_alphabet_index + 1;

            let pair_count = self.stream.read_symbol(&self.natural8_usize_table)?;
            let mut pairs: Vec<(SymbolArrayIndex, SymbolArrayIndex)> = Vec::with_capacity(pair_count);
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
        let number_of_correlations = self.stream.read_symbol(&self.natural8_usize_table)?;
        let mut correlations: Vec<HashMap<Alphabet, SymbolArrayIndex>> = Vec::with_capacity(number_of_correlations);
        if number_of_correlations > 0 {
            // The serialization of correlations can be improved in several ways:
            // - There can be only one correlation with length 0. It could be serialised with a single bit: 0 (not present), 1 (present at the beginning)
            // - If correlations cannot mix alphabets from different languages, then we could reduce the number of possible keys once we know the first key, or even the language. For languages where only one alphabet is available, then the length and the key gets irrelevant
            // TODO: Improve codification for this table, it include lot of edge cases that should not be possible
            let length_table = self.stream.read_table(&self.integer8_table, &self.natural8_table, InputBitStream::read_symbol,InputBitStream::read_diff_i32)?;
            for _ in 0..number_of_correlations {
                let map_length = usize::try_from(self.stream.read_symbol(&length_table)?).unwrap();
                if map_length >= alphabet_count {
                    panic!("Map for correlation cannot be longer than the actual number of valid alphabets");
                }

                let mut map: HashMap<Alphabet, SymbolArrayIndex> = HashMap::with_capacity(map_length);
                if map_length > 0 {
                    let key_table = RangedNaturalUsizeHuffmanTable::new(0, alphabet_count - map_length);
                    let value_table = RangedNaturalUsizeHuffmanTable::new(0, symbol_array_count - 1);
                    let mut raw_key = self.stream.read_symbol(&key_table)?;
                    let key = Alphabet {
                        index: raw_key
                    };

                    let value = SymbolArrayIndex {
                        index: self.stream.read_symbol(&value_table)?
                    };
                    map.insert(key, value);
                    for map_index in 1..map_length {
                        let key_diff_table = RangedNaturalUsizeHuffmanTable::new(raw_key + 1, alphabet_count - map_length + map_index);
                        raw_key = self.stream.read_symbol(&key_diff_table)?;
                        let key = Alphabet {
                            index: raw_key
                        };

                        let value = SymbolArrayIndex {
                            index: self.stream.read_symbol(&value_table)?
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
        let number_of_arrays = self.stream.read_symbol(&self.natural8_usize_table)?;
        let mut arrays: Vec<Vec<CorrelationIndex>> = Vec::with_capacity(number_of_arrays);
        if number_of_arrays > 0 {
            let correlation_table = RangedNaturalUsizeHuffmanTable::new(0, number_of_correlations - 1);
            // TODO: Improve codification for this table, it include lot of edge cases that should not be possible
            let length_table = self.stream.read_table(&self.integer8_table, &self.natural8_table, InputBitStream::read_symbol,InputBitStream::read_diff_i32)?;

            for _ in 0..number_of_arrays {
                let array_length = usize::try_from(self.stream.read_symbol(&length_table)?).unwrap();
                let mut array: Vec<CorrelationIndex> = Vec::with_capacity(array_length);
                for _ in 0..array_length {
                    array.push(CorrelationIndex {
                        index: self.stream.read_symbol(&correlation_table)?
                    });
                }
                arrays.push(array);
            }
        }

        Ok(arrays)
    }

    fn read_acceptations(&mut self, min_valid_concept: usize, max_valid_concept: usize, correlation_array_count: usize) -> Result<Vec<Acceptation>, ReadError> {
        let number_of_entries = self.stream.read_symbol(&self.natural8_usize_table)?;
        let mut result: Vec<Acceptation> = Vec::new();
        if number_of_entries > 0 {
            // TODO: Improve codification for this table, it include some edge cases that should not be possible, like negative values for lengths
            let correlation_array_set_length_table = self.stream.read_table(&self.integer8_table, &self.natural8_table, InputBitStream::read_symbol, InputBitStream::read_diff_i32)?;
            let concept_table = RangedNaturalUsizeHuffmanTable::new(min_valid_concept, max_valid_concept);
            for _ in 0..number_of_entries {
                let concept = self.stream.read_symbol(&concept_table)?;
                let length = usize::try_from(self.stream.read_symbol(&correlation_array_set_length_table)?).unwrap();
                let symbol_table = RangedNaturalUsizeHuffmanTable::new(0, correlation_array_count - length);
                let mut value = self.stream.read_symbol(&symbol_table)?;
                result.push(Acceptation {
                    concept,
                    correlation_array_index: CorrelationArrayIndex {
                        index: usize::try_from(value).unwrap()
                    }
                });

                for set_entry_index in 1..length {
                    let symbol_diff_table = RangedNaturalUsizeHuffmanTable::new(value + 1, correlation_array_count - length + set_entry_index);
                    value += self.stream.read_symbol(&symbol_diff_table)? + 1;
                    result.push(Acceptation {
                        concept,
                        correlation_array_index: CorrelationArrayIndex {
                            index: value
                        }
                    });
                }
            }
        }

        Ok(result)
    }

    fn read_definitions(&mut self, min_valid_concept: usize, max_valid_concept: usize) -> Result<HashMap<usize, Definition>, ReadError> {
        let number_of_base_concepts = self.stream.read_symbol(&self.natural8_usize_table)?;
        let mut definitions: HashMap<usize, Definition> = HashMap::new();
        if number_of_base_concepts > 0 {
            let concept_map_length_table = self.stream.read_table(&self.natural8_table, &self.natural8_table, InputBitStream::read_symbol, InputBitStream::read_diff_u32)?;
            let mut min_base_concept = min_valid_concept;
            for max_base_concept in (max_valid_concept - number_of_base_concepts + 1)..=max_valid_concept {
                let table = RangedNaturalUsizeHuffmanTable::new(min_base_concept, max_base_concept);
                let base = self.stream.read_symbol(&table)?;
                min_base_concept = base + 1;

                let map_length = usize::try_from(self.stream.read_symbol(&concept_map_length_table)?).unwrap();
                if map_length > 0 {
                    let concept_table = RangedNaturalUsizeHuffmanTable::new(min_valid_concept, max_valid_concept - map_length + 1);
                    let mut concept = self.stream.read_symbol(&concept_table)?;

                    fn read_complements(stream: &mut InputBitStream, min_valid_concept: usize, max_valid_concept: usize) -> Result<HashSet<usize>, ReadError> {
                        let mut min_valid_complement = min_valid_concept;
                        let mut complements: HashSet<usize> = HashSet::new();
                        while min_valid_complement < max_valid_concept && stream.read_boolean()? {
                            let complement_table = RangedNaturalUsizeHuffmanTable::new(min_valid_complement, max_valid_concept);
                            let complement = stream.read_symbol(&complement_table)?;
                            min_valid_complement = complement + 1;
                            complements.insert(complement);
                        }

                        Ok(complements)
                    }

                    definitions.insert(concept, Definition {
                        base_concept: base,
                        complements: read_complements(&mut self.stream, min_valid_concept, max_valid_concept)?
                    });

                    for map_index in 1..map_length {
                        let concept_table = RangedNaturalUsizeHuffmanTable::new(concept + 1, max_valid_concept - map_length + 1 + map_index);
                        concept = self.stream.read_symbol(&concept_table)?;

                        definitions.insert(concept, Definition {
                            base_concept: base,
                            complements: read_complements(&mut self.stream, min_valid_concept, max_valid_concept)?
                        });
                    }
                }
            }
        }

        Ok(definitions)
    }

    pub fn read(mut self) -> Result<SdbReadResult, ReadError> {
        let symbol_array_count = self.stream.read_symbol(&self.natural8_usize_table)?;
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
        let max_concept = self.stream.read_symbol(&self.natural8_usize_table)?;
        let correlations = self.read_correlations(alphabet_count, symbol_array_count)?;
        let correlation_arrays = self.read_correlation_arrays(correlations.len())?;
        let acceptations = self.read_acceptations(1, max_concept, correlation_arrays.len())?;
        let definitions = self.read_definitions(1, max_concept)?;

        Ok(SdbReadResult {
            symbol_arrays,
            languages,
            conversions,
            max_concept,
            correlations,
            correlation_arrays,
            acceptations,
            definitions
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
