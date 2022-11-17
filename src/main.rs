use std::env;
use std::fs::File;
use std::io::{Bytes, Read};

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

struct ReadError {
    message: String
}

impl ReadError {
    fn new(message: &str) -> ReadError {
        ReadError {
            message: message.to_string()
        }
    }
}

fn read_u8(bytes: &mut Bytes<File>) -> Result<u8, ReadError> {
    match bytes.next() {
        None => Err(ReadError::new("Unexpected end of file")),
        Some(result) => match result {
            Err(err) => Err(ReadError::new(&err.to_string())),
            Ok(x) => Ok(x)
        }
    }
}

fn assert_next_is_same_u8(bytes: &mut Bytes<File>, value: u8) -> Result<bool, ReadError> {
    match read_u8(bytes) {
        Err(x) => Err(x),
        Ok(x) => {
            if x == value {
                Ok(true)
            }
            else {
                Err(ReadError::new(&format!("Unexpected character 0x{:X}, expectation was 0x{:X}", x, value)))
            }
        }
    }
}

fn assert_next_is_same_text(bytes: &mut Bytes<File>, text: &str) -> Result<bool, ReadError> {
    for expected_value in text.bytes() {
        assert_next_is_same_u8(bytes, expected_value)?;
    }

    return Ok(true)
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
                    match assert_next_is_same_text(&mut bytes, "SDB\x01") {
                        Ok(_) => println!("All fine so far"),
                        Err(err) => println!("Error found: {}", err.message)
                    }
                }
            }
        }
    }
}
