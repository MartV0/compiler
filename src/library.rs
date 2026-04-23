use crate::abstract_syntax_tree::Program;
use crate::parsing::parse;

use nom::error::Error;
use nom::Err;

/// Add library to the program
pub fn add_library(program: &mut Program) {
    let lib = include_str!("../poo_lib/lib.poo");
    let parsed_lib: Result<_, Err<Error<_>>> = parse(lib);
    let Program { mut functions, mut variables } = parsed_lib.expect("Failed to parse lib");
    program.functions.append(&mut functions);
    program.variables.append(&mut variables);
}
