mod parsing;
mod abstract_syntax_tree;
mod linking;
mod compiling;
mod test_compilation;
mod assembling;

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
    print_compiled(&compiled_program);
    let assembled_program = assembling::assemble(compiled_program);
    print_vecu8(&assembled_program.code);
    let binary = linking::elf::create_elf(assembled_program);
    fs::write(out_file_path, binary).expect("failed to write to file");
}

fn print_vecu8(input: &Vec<u8>) {
    for val in input {
        print!("{val:#04x} ");
    }
    println!("");
}

fn print_compiled(program: &compiling::CompilationResult) {
    println!("CODE:");
    for instruction in program.code.iter() {
        println!("{instruction:?}");
    }
    println!("\nDATA:");
    for data in program.data.iter() {
        println!("{data:?}");
    }
}
