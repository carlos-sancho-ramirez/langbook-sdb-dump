use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Arguments found:");
    let mut next_is_input = false;
    let mut input_file_name = String::new();
    let mut input_file_name_set = false;
    let mut is_first = true;
    for arg in &args {
        if is_first {
            is_first = false;
        }
        else if next_is_input {
            next_is_input = false;
            input_file_name_set = true;
            input_file_name.push_str(arg);
        }
        else if arg == "-i" {
            if input_file_name_set {
                panic!("Input file already set");
            }
            else {
                next_is_input = true
            }
        }
        else {
            panic!("Invalid argument {}", arg);
        }
    }

    if input_file_name_set {
        println!("Reading file {}", input_file_name);
    }
    else {
        panic!("Missing input file: try {} -i <sdb-file>", &args[0]);
    }
}
