use std::fs::File;
use std::io::Bytes;

pub struct ReadError {
    pub message: String
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

pub fn assert_next_is_same_text(bytes: &mut Bytes<File>, text: &str) -> Result<bool, ReadError> {
    for expected_value in text.bytes() {
        assert_next_is_same_u8(bytes, expected_value)?;
    }

    return Ok(true)
}
