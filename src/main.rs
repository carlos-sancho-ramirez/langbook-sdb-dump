use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Arguments found:");
    for arg in args {
        println!("  {}", arg);
    }
}
