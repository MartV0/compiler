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
}

#[derive(Debug)]
pub enum Operand {
    Immediate(ImmediateValue),
    Register(Register),
    Indirect(Register),
}

#[derive(Debug)]
pub enum ImmediateValue {
    Literal(u64),
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

/// Assembles the code into raw bytecode and data segment, still needs to be converted to elf/linked
pub fn assemble(code: CompilationResult) -> AssemblingResult {
    // Create the data section and labels
    let CompilationResult { code, data } = code;
    let mut data_labels = HashMap::new();
    let mut data_section = vec![];
    for (label, mut data) in data.into_iter() {
        data_labels.insert(
            label,
            data_section.len().try_into().unwrap()
        );
        data_section.append(&mut data);
    }

    let mut output = AssemblingResult {
        code: vec![],
        data: data_section,
        code_relocate: vec![],
    };
    assemble_code(code, data_labels, &mut output);

    output
}

/// Assemble the instruction into their actual bytecode
fn assemble_code(code: Vec<Instruction>, data_labels: HashMap<String, u64>, output: &mut AssemblingResult) {
    for instruction in code {
        match instruction {
            Instruction::Mov(Operand::Register(reg), Operand::Immediate(val))
                if is_32bit_reg(&reg) | is_64bit_reg(&reg) =>
            {
                create_instruction_modrm(
                    output,
                    vec![0xC7],
                    reg,
                    None,
                    val,
                    4,
                    &data_labels
                );
            }
            Instruction::Syscall => output.code.append(&mut vec![0x0F, 0x05]),
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

/// Create a instruction that has a MOD R/M argument
/// - rm: r/m register in the mod reg r/m byte
/// - reg: reg argument in the mod reg r/m byte
/// - immediate: the immediate value operant
/// - immediate_bytes: how much bytes the immediate operant is
/// - data_labels: hashmap that converts from data labels to addresses
// TODO: offset + offset_bytes
fn create_instruction_modrm(
    output: &mut AssemblingResult,
    mut opcode: Vec<u8>,
    rm: Register,
    reg: Option<Register>,
    immediate: ImmediateValue,
    immediate_bytes: u8,
    data_labels: &HashMap<String, u64>,
) {
    let rm_bits = reg_to_XREG_bits(&rm);
    // TODO: reg can optionally contain opcode extension
    let reg_bits = reg.map_or(0, |reg| reg_to_XREG_bits(&reg));

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

    let immediate = match immediate {
        ImmediateValue::Literal(bytes) => bytes,
        ImmediateValue::Label(label, segment_type) => {
            output.code_relocate.push(relocate::RelocationEntrie {
                offset: output.code.len(),
                bytes: immediate_bytes,
                segment: segment_type,
            });
            *data_labels.get(&label).expect("undefined label referenced")
        }
    };

    for val in immediate.to_le_bytes().into_iter().take(immediate_bytes.into()) {
        output.code.push(val);
    }
}

/// returns 4 bit representation of register
/// most significant of the 4 bits is for REX.R other 3 for MOD R/M .REG
#[rustfmt::skip]
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
