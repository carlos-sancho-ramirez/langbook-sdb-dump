use std::env;

fn main() {
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
                panic!("Input file already set");
            }
        }
        else {
            panic!("Invalid argument {}", arg);
        }
    }

    match input_file_name {
        Some(name) => println!("Reading file {}", name),
        None => panic!("Missing input file: try {} -i <sdb-file>", env::args().next().expect("wtf?"))
    }
}
