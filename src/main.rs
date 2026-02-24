mod parsing;
mod abstract_syntax_tree;

use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    let file_contents = fs::read_to_string(file_path)
        .expect("Failed to read input code file");
    let parsed_program = parsing::parse(&file_contents);
    println!("{parsed_program:?}");
}
