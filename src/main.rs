mod parsing;
mod abstract_syntax_tree;
mod linking;
mod compiling;
mod test_compilation;

use std::path::Path;
use std::{env, fs};
use nom::error::Error;
use nom::Err;

fn main() {
    let args: Vec<String> = env::args().collect();
    let in_file_path = &args[1];
    let in_file_contents = fs::read_to_string(in_file_path)
        .expect("Failed to read input code file");
    let out_file_path = Path::new(&args[2]);
    compile(&in_file_contents, out_file_path);
}

fn compile(program: &str, out_file_path: &Path) {
    let parsed_program: Result<_, Err<Error<_>>> = parsing::parse(&program);
    let compiled_program = compiling::compile(parsed_program.expect("failed to parse program"));
    let binary = linking::elf::create_elf(compiled_program);
    fs::write(out_file_path, binary).expect("failed to write to file");
}
