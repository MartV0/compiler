use crate::linking::elf::SegmentType;

use Register::*;

#[derive(Debug, Clone)]
pub enum Instruction {
    // Not an actual instruction, just used to point to a location in the code
    ILabel(Label),
    Mov(Operand, Operand),
    // Mov with zero extent, for when moving smaller sized operand to bigger sized one
    MovZX(Operand, Operand),
    Syscall,
    Jmp(Operand),
    JE(ImmediateValue),
    JNE(ImmediateValue),
    Pop(Operand),
    Push(Operand),
    Call(Operand),
    Leave,
    Ret,
    Sub(Operand, Operand),
    Cmp(Operand, Operand),
    Add(Operand, Operand),
    /// Signed multiplication
    IMul(Operand, Operand),
    /// Signed division: EDX:EAX / operand
    IDiv(Operand),
    // Bitwise and
    And(Operand, Operand),
    // Bitwise or
    Or(Operand, Operand),
    // Bitwise xor
    Xor(Operand, Operand),
    // Bitwise not
    Not(Operand),
    /// Conditionals set bytes
    SetLE(Operand),
    SetL(Operand),
    SetGE(Operand),
    SetG(Operand),
    SetE(Operand),
    SetNE(Operand),
    /// Computes the effective address of the second operand (the source operand) and stores it in the first operand (destination operand)
    LEA(Operand, Operand),
}

#[derive(Debug, Clone)]
pub enum Operand {
    Immediate(ImmediateValue),
    IndirectDisplacement(Register, i32),
    Register(Register),
    Indirect(Register),
}

#[derive(Debug, Clone)]
pub enum ImmediateValue {
    Literal(i64),
    Label(Label, SegmentType),
}

pub type Label = String;

#[allow(dead_code)]
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq)]
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

/// Check if reg is a 64 bit register
#[rustfmt::skip]
pub fn is_64bit_reg(reg: &Register) -> bool {
    match reg {
        RAX | RBX | RCX | RDX | RSI | RDI | RSP |
        RBP | R8  | R9  | R10 | R11 | R12 | R13 |
        R14 | R15 => true,
        _ => false
    }
}

/// Check if reg is a 32 or 64 bit register
#[rustfmt::skip]
pub fn is_32or64_bit_reg(reg: &Register) -> bool {
    is_32bit_reg(reg) || is_64bit_reg(reg)
}

/// Check if reg is a 32 bit register
#[rustfmt::skip]
pub fn is_32bit_reg(reg: &Register) -> bool {
    match reg {
        EAX  | EBX  | ECX  | EDX  | ESI | EDI | ESP | EBP | R8D | R9D | R10D |
        R11D | R12D | R13D | R14D | R15D => true,
        _ => false,
    }
}

/// Check if reg is a 8 bit register
#[rustfmt::skip]
pub fn is_8bit_reg(reg: &Register) -> bool {
    match reg {
        AH  | AL  | BH  | BL   | CH   | CL   | DH   | DL   | SIL | DIL | SPL |
        BPL | R8B | R9B | R10B | R11B | R12B | R13B | R14B | R15B => true,
        _ => false,
    }
}
