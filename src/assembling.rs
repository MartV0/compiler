// References:
// - Doesn't contain x86_64 but still great guide/introduction on instruction encoding https://www.c-jump.com/CIS77/CPU/x86/lecture.html#X77_0010_real_encoding
// - With x86_64 but not great as learning resource, better as quick reference: https://wiki.osdev.org/X86-64_Instruction_Encoding

use std::collections::HashMap;

use crate::compiling::CompilationResult;
use crate::linking::elf::SegmentType;
use crate::linking::relocate;

use Register::*;

#[derive(Debug)]
pub enum Instruction {
    // Not an actual instruction, just used to point to a location in the code
    ILabel(Label),
    Mov(Operand, Operand),
    Syscall,
    Jmp(Operand),
    Pop(Operand),
    PushI(ImmediateValue, OperandSize),
    PushR(Register),
    Call(Operand),
    Leave,
    Ret,
    Sub(Operand, Operand),
    Add(Operand, Operand),
}

#[derive(Debug)]
pub enum Operand {
    Immediate(ImmediateValue),
    Offset(ImmediateValue),
    Register(Register),
    Indirect(Register),
}

// Sometimes explicit operand size is needed
// For example push, when there is no register operand, something else is
// needed to figure out how much bytes to push
pub type OperandSize = u8;

#[derive(Debug)]
pub enum ImmediateValue {
    // TODO: signed immediate value
    Literal(i64),
    Label(Label, SegmentType),
}

pub type Label = String;

#[allow(dead_code)]
#[rustfmt::skip]
#[derive(Debug)]
pub enum Register {
    RAX, EAX,  AX,   AH,   AL,
    RBX, EBX,  BX,   BH,   BL,
    RCX, ECX,  CX,   CH,   CL,
    RDX, EDX,  DX,   DH,   DL,
    RSI, ESI,  SI,         SIL,
    RDI, EDI,  DI,         DIL,
    RSP, ESP,  SP,         SPL,
    RBP, EBP,  BP,         BPL,
    R8,  R8D,  R8W,        R8B,
    R9,  R9D,  R9W,        R9B,
    R10, R10D, R10W,       R10B,
    R11, R11D, R11W,       R11B,
    R12, R12D, R12W,       R12B,
    R13, R13D, R13W,       R13B,
    R14, R14D, R14W,       R14B,
    R15, R15D, R15W,       R15B,
}

/// Struct containing the raw bytecode and data, still needs to be converted to elf/linked
#[derive(Debug)]
pub struct AssemblingResult {
    pub code: Vec<u8>,
    pub data: Vec<u8>,
    pub code_relocate: Vec<relocate::RelocationEntrie>,
}

/// Only used internally, contains some extra data used during compilation, but
/// not needed in the final result
struct IntermediateAssemblingResult {
    code: Vec<u8>,
    data: Vec<u8>,
    code_relocate: Vec<relocate::RelocationEntrie>,
    data_labels: HashMap<String, u64>,
    code_labels: HashMap<String, u64>,
    code_label_positions: Vec<CodeLabelPosition>,
}

// it is possible we encounter a code label before it's position is now,
// therefore we keep track of a seperate list, where these are fixed later
struct CodeLabelPosition {
    label: String,
    // index where the address is
    address_index: usize,
    label_type: LabelType,
    bytes: u8,
}

enum LabelType {
    // index where the next instruction is, because the jump value is relative to
    // the next instruction
    Relative,
    Absolute
}

/// Assembles the code into raw bytecode and data segment, still needs to be converted to elf/linked
pub fn assemble(code: CompilationResult) -> AssemblingResult {
    // Create the data section and labels
    let CompilationResult { code, data } = code;
    let mut result = IntermediateAssemblingResult {
        code: vec![],
        data: vec![],
        code_relocate: vec![],
        data_labels: HashMap::new(),
        code_labels: HashMap::new(),
        code_label_positions: vec![],
    };
    for (label, mut data) in data.into_iter() {
        result
            .data_labels
            .insert(label, result.data.len().try_into().unwrap());
        result.data.append(&mut data);
    }

    assemble_code(code, &mut result);

    fix_code_labels(&mut result);

    AssemblingResult {
        code: result.code,
        data: result.data,
        code_relocate: result.code_relocate,
    }
}

/// Assemble the instruction into their actual bytecode
fn assemble_code(code: Vec<Instruction>, output: &mut IntermediateAssemblingResult) {
    for instruction in code {
        match instruction {
            Instruction::ILabel(label) => {
                output
                    .code_labels
                    .insert(label, output.code.len().try_into().unwrap());
            }
            Instruction::Mov(Operand::Register(reg), Operand::Immediate(val))
                if is_32bit_reg(&reg) | is_64bit_reg(&reg) =>
            {
                add_rex_opcode_modrm(output, vec![0xC7], reg, RegValue::None);
                add_immediate(output, val, 4);
            }
            Instruction::Mov(Operand::Register(rm), Operand::Register(reg)) => {
                add_rex_opcode_modrm(output, vec![0x89], rm, RegValue::Register(reg));
            }
            Instruction::Syscall => output.code.append(&mut vec![0x0F, 0x05]),
            Instruction::Leave => output.code.push(0xC9),
            Instruction::Ret => output.code.push(0xC3),
            Instruction::Call(Operand::Immediate(immediate)) => {
                let code_index = output.code.len();
                output.code.push(0xE8);
                add_offset(
                    output,
                    immediate,
                    4,
                    LabelType::Relative
                );
            }
            Instruction::Jmp(Operand::Immediate(immediate)) => {
                let code_index = output.code.len();
                output.code.push(0xE9);
                add_offset(
                    output,
                    immediate,
                    4,
                    LabelType::Relative,
                );
            }
            Instruction::PushR(reg) => {
                add_rex_opcode_modrm64bit(output, vec![0xFF], reg, RegValue::Extension(0x06));
            }
            Instruction::Pop(Operand::Register(reg)) if is_64bit_reg(&reg) => {
                add_rex_opcode_modrm(output, vec![0x8F], reg, RegValue::None);
            }
            Instruction::PushI(immediate, size) if size == 1 || size == 4 => {
                output.code.push(match size {
                    1 => 0x6A,
                    4 => 0x68,
                    _ => panic!("Invalid push immediate size")
                });
                add_immediate(output, immediate, size);
            }
            val => panic!("unsupported instruction: {val:?}"),
        }
    }
}

/// Check reg is a 64 bit register
#[rustfmt::skip]
fn is_64bit_reg(reg: &Register) -> bool {
    match reg {
        RAX | RBX | RCX | RDX | RSI | RDI | RSP |
        RBP | R8  | R9  | R10 | R11 | R12 | R13 |
        R14 | R15 => true,
        _ => false
    }
}

/// Check reg is a 32 bit register
#[rustfmt::skip]
fn is_32bit_reg(reg: &Register) -> bool {
    match reg {
        EAX  | EBX  | ECX  | EDX  | ESI | EDI | ESP | EBP | R8D | R9D | R10D |
        R11D | R12D | R13D | R14D | R15D => true,
        _ => false,
    }
}

/// same as add_rex_opcode_modrm but assumes 64 bit size operants,
/// meaning rex.w byte is not needed
fn add_rex_opcode_modrm64bit(
    output: &mut IntermediateAssemblingResult,
    mut opcode: Vec<u8>,
    rm: Register,
    reg: RegValue,
) {
    // Just converts the 64 bit register to 32 bit ones and use regular function
    let reg = match reg {
        RegValue::Register(register) => RegValue::Register(reg64_to_reg32(register)),
        x => x,
    };

    add_rex_opcode_modrm(output, opcode, reg64_to_reg32(rm), reg);
}

fn reg64_to_reg32(reg: Register) -> Register {
    match reg {
        RAX => EAX, 
        RBX => EBX, 
        RCX => ECX, 
        RDX => EDX, 
        RSI => ESI, 
        RDI => EDI, 
        RSP => ESP, 
        RBP => EBP, 
        R8  =>  R8D,
        R9  =>  R9D,
        R10 => R10D,
        R11 => R11D,
        R12 => R12D,
        R13 => R13D,
        R14 => R14D,
        R15 => R15D,
        reg => reg
    }
}

enum RegValue {
    Register(Register),
    Extension(u8),
    None
}
/// Create a instruction that has a MOD R/M argument
/// - rm: r/m register in the mod reg r/m byte
/// - reg: reg argument in the mod reg r/m byte
/// - immediate: the immediate value operant
/// - immediate_bytes: how much bytes the immediate operant is
/// - data_labels: hashmap that converts from data labels to addresses
/// - opcode_extension: extend the upcode with a reg
// TODO: offset + offset_bytes
fn add_rex_opcode_modrm(
    output: &mut IntermediateAssemblingResult,
    mut opcode: Vec<u8>,
    rm: Register,
    reg: RegValue,
) {
    let rm_bits = reg_to_XREG_bits(&rm);
    // TODO: reg can optionally contain opcode extension
    let reg_bits = match reg {
        RegValue::Register(reg) => reg_to_XREG_bits(&reg),
        RegValue::Extension(ext) => ext,
        RegValue::None => 0,
    };

    let rex_w = is_64bit_reg(&rm);
    let rex_r = (reg_bits >> 3) == 1;
    // TODO: SIB.index
    let rex_x = false;
    let rex_b = (rm_bits >> 3) == 1;
    let rex: u8 = 0b0100 << 4
        | to_byte(rex_w) << 3
        | to_byte(rex_r) << 2
        | to_byte(rex_x) << 1
        | to_byte(rex_b);
    // rex only needed if it actually encodes anything
    if rex_w | rex_r | rex_x | rex_b {
        output.code.push(rex);
    }

    output.code.append(&mut opcode);

    // TODO: addressing modes
    let mod_ = 0b11;
    let reg = reg_bits & 0b111;
    let rm = rm_bits & 0b111;
    let mod_reg_rm = mod_ << 6 | reg << 3 | rm;

    output.code.push(mod_reg_rm);
}

/// Adds the immediate part to the instruction
fn add_immediate(
    output: &mut IntermediateAssemblingResult,
    immediate: ImmediateValue,
    immediate_bytes: u8,
) {
    add_offset(output, immediate, immediate_bytes, LabelType::Absolute);
}

/// Adds the offset part to the instruction
fn add_offset(
    output: &mut IntermediateAssemblingResult,
    offset: ImmediateValue,
    offset_bytes: u8,
    label_type: LabelType,
) {
    let immediate = match offset {
        ImmediateValue::Literal(bytes) => bytes.to_le_bytes(),
        ImmediateValue::Label(label, segment_type) => {
            // Relocation only needed if it is a absolute offset
            if let LabelType::Absolute = label_type {
                output.code_relocate.push(relocate::RelocationEntrie {
                    offset: output.code.len(),
                    bytes: offset_bytes,
                    segment: segment_type.clone(),
                });
            }
            match segment_type {
                SegmentType::Data => output
                    .data_labels
                    .get(&label)
                    .expect("undefined label referenced")
                    .to_le_bytes(),
                SegmentType::Text => {
                    output.code_label_positions.push(CodeLabelPosition {
                        label,
                        address_index: output.code.len(),
                        label_type,
                        bytes: offset_bytes,
                    });
                    [0; 8]
                }
            }
        }
    };

    for val in immediate.into_iter().take(offset_bytes.into()) {
        output.code.push(val);
    }
}

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

/// returns 4 bit representation of register
/// most significant of the 4 bits is for REX.R other 3 for MOD R/M .REG
#[rustfmt::skip]
#[allow(non_snake_case)]
fn reg_to_XREG_bits(register: &Register) -> u8 {
    match register {
        RAX | EAX  |  AX  |       AL => 0b0000,
        RCX | ECX  |  CX  |       CL => 0b0001,
        RDX | EDX  |  DX  |       DL => 0b0010,
        RBX | EBX  |  BX  |       BL => 0b0011,
        RSP | ESP  |  SP  | AH | SPL => 0b0100,
        RBP | EBP  |  BP  | CH | BPL => 0b0101,
        RSI | ESI  |  SI  | DH | SIL => 0b0110,
        RDI | EDI  |  DI  | BH | DIL => 0b0111,
        R8  | R8D  |  R8W |      R8B => 0b1000,
        R9  | R9D  |  R9W |      R9B => 0b1001,
        R10 | R10D | R10W |     R10B => 0b1010,
        R11 | R11D | R11W |     R11B => 0b1011,
        R12 | R12D | R12W |     R12B => 0b1100,
        R13 | R13D | R13W |     R13B => 0b1101,
        R14 | R14D | R14W |     R14B => 0b1110,
        R15 | R15D | R15W |     R15B => 0b1111,
    }
}

/// Converts byte to u8, added because otherwise .into() required annotation everywhere
fn to_byte(val: bool) -> u8 {
    val.into()
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
            //	sys_read system call
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
