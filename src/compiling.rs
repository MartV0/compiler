use std::collections::HashMap;
use crate::abstract_syntax_tree;
use crate::linking::elf::SegmentType;
use crate::assembling::{
    Instruction::*,
    Instruction,
    Operand::*,
    ImmediateValue::*,
    Register::*,
};

/// Struct containing the raw bytecode and data, still needs to be converted to elf/linked
pub struct CompilationResult {
    pub code: Vec<Instruction>,
    pub data: HashMap<crate::assembling::Label, Vec<u8>>,
}

/// Generates bytecode section, and string section from AST
pub fn compile(program: abstract_syntax_tree::Program) -> CompilationResult {
    let hello: Vec<u8> = "Hello world\n".as_bytes().to_vec();
    let hello_len: u64 = hello.len().try_into().expect("Failed to convert usize to u64");
    let hello_label = "hello_string".to_string();
    let code: Vec<Instruction> = vec![
        //	sys_read system call
        Mov(Register(EAX), Immediate(Literal(0x1))),
        //	file descriptor = stdout
        Mov(Register(EDI), Immediate(Literal(0x1))),
        //	char* = pointer to string
        Mov(Register(RSI), Immediate(Label(hello_label.clone(), SegmentType::Data))),
        //	mov    edx, length of string
        Mov(Register(EDX), Immediate(Literal(hello_len))),
        //	syscall
        Syscall,
        //	sys_exit system call
        Mov(Register(EAX), Immediate(Literal(0x3c))),
        //	exit code: 0x0
        Mov(Register(EDI), Immediate(Literal(0))),
        //	syscall
        Syscall,
    ];

    let data = HashMap::from([
        (hello_label, hello)
    ]);

    CompilationResult {
        code,
        data,
    }
}
