mod parsing;
mod abstract_syntax_tree;
mod linking;

use std::{env, fs};
use nom::error::Error;
use nom::Err;

fn main() {
    let args: Vec<String> = env::args().collect();
    let in_file_path = &args[1];
    let in_file_contents = fs::read_to_string(in_file_path)
        .expect("Failed to read input code file");
    let out_file_path = &args[2];
    let parsed_program: Result<_, Err<Error<_>>> = parsing::parse(&in_file_contents);
    let binary = linking::elf::create_elf();
    fs::write(out_file_path, binary).expect("failed to write to file");
    println!("{parsed_program:?}");
}
