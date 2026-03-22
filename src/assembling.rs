// In this module and submodules the assembly gets turned into bytecode
// References:
// - Doesn't contain x86_64 but still great guide/introduction on instruction encoding https://www.c-jump.com/CIS77/CPU/x86/lecture.html#X77_0010_real_encoding
// - With x86_64 but not great as learning resource, better as quick reference: https://wiki.osdev.org/X86-64_Instruction_Encoding
pub mod assembly;
pub mod assemble_instruction_part;
pub mod assemble_instruction;

use std::collections::HashMap;

use crate::compiling::CompilationResult;
use crate::linking::elf::SegmentType;
use crate::linking::relocate;

use assembly::Register::*;
use assembly::*;

/// Struct containing the raw bytecode and data, still needs to be converted to elf/linked
#[derive(Debug)]
pub struct AssemblingResult {
    pub code: Vec<u8>,
    pub data: Vec<u8>,
    pub code_relocate: Vec<relocate::RelocationEntrie>,
}

/// Only used internally, contains some extra data used during compilation, but
/// not needed in the final result
pub struct IntermediateAssemblingResult {
    code: Vec<u8>,
    data: Vec<u8>,
    code_relocate: Vec<relocate::RelocationEntrie>,
    data_labels: HashMap<String, u64>,
    code_labels: HashMap<String, u64>,
    code_label_positions: Vec<CodeLabelPosition>,
}

// It is possible we encounter a code label before it's position is known,
// so we fill in the label as a final compilation step
struct CodeLabelPosition {
    label: String,
    // index where the address is
    address_index: usize,
    label_type: LabelType,
    bytes: u8,
}

/// Whether a label is relative to next instruction pointer or absolute
pub enum LabelType {
    // index where the next instruction is, because the jump value is relative to
    // the next instruction
    Relative,
    Absolute
}

/// Assembles the code into raw bytecode and data segment, still needs to be converted to elf/linked
pub fn assemble(code: CompilationResult) -> AssemblingResult {
    let CompilationResult { code, data } = code;
    let mut result = IntermediateAssemblingResult {
        code: vec![],
        data: vec![],
        code_relocate: vec![],
        data_labels: HashMap::new(),
        code_labels: HashMap::new(),
        code_label_positions: vec![],
    };

    create_data_section(&mut result, &data);

    assemble_code(code, &mut result);

    fix_code_labels(&mut result);

    AssemblingResult {
        code: result.code,
        data: result.data,
        code_relocate: result.code_relocate,
    }
}

/// Creates the data section and labels
fn create_data_section(output: &mut IntermediateAssemblingResult, data: &HashMap<String, Vec<u8>>) {
    for (label, data) in data.into_iter() {
        output
            .data_labels
            .insert(label.clone(), output.data.len().try_into().unwrap());
        for byte in data {
            output.data.push(*byte);
        }
    }
}

/// Assemble the instruction into their actual bytecode
fn assemble_code(code: Vec<Instruction>, output: &mut IntermediateAssemblingResult) {
    for instruction in code {
        assemble_instruction::assemble_instruction(instruction, output);
    }
}

/// Fixes label based addresses in the code, this is done in a seperate step
/// because it's possible we encounter a code label before it's position is known
fn fix_code_labels(output: &mut IntermediateAssemblingResult) {
    for CodeLabelPosition {
        label,
        address_index,
        label_type,
        bytes,
    } in output.code_label_positions.iter()
    {
        let bytes = (*bytes).into();
        let offset = *address_index;
        let label_adress: i64 = (*output.code_labels.get(label).expect("Could not find label")).try_into().unwrap();

        let new_address: i64 = match label_type {
            LabelType::Relative => {
                let next_instruction: i64 = usize::try_into(offset + bytes).unwrap();
                label_adress - next_instruction
            },
            LabelType::Absolute => label_adress,
        };

        // Write the new address back as little endian bytes
        output.code[offset..offset + bytes].clone_from_slice(&new_address.to_le_bytes()[0..bytes]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembling::{
        ImmediateValue::*,
        Instruction::{self, *},
        Operand::*,
    };
    use crate::linking::elf::SegmentType;
    use crate::test_compilation::test_assembler;
    use std::collections::HashMap;

    #[test]
    fn test_hello_world() {
        let hello: Vec<u8> = "Hello world\n".as_bytes().to_vec();
        let hello_len: i64 = hello
            .len()
            .try_into()
            .expect("Failed to convert usize to u64");
        let hello_label = "hello_string".to_string();
        let code: Vec<Instruction> = vec![
            //	sys_write system call
            Mov(Register(EAX), Immediate(Literal(0x1))),
            //	file descriptor = stdout
            Mov(Register(EDI), Immediate(Literal(0x1))),
            //	char* = pointer to string
            Mov(
                Register(RSI),
                Immediate(Label(hello_label.clone(), SegmentType::Data)),
            ),
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

        let data = HashMap::from([(hello_label, hello)]);

        let compiled = CompilationResult { code, data };

        let std::process::Output {
            status,
            stdout,
            stderr,
        } = test_assembler(compiled).unwrap();
        assert_eq!(status.code(), Some(0));
        assert_eq!(stdout, "Hello world\n".as_bytes().to_vec());
        assert_eq!(stderr, vec![]);
    }
}
