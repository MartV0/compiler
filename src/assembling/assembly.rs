use crate::linking::elf::SegmentType;

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

/// Sometimes explicit operand size is needed
/// For example with push, when there is no register operand, something else is
/// needed to figure out how much bytes to push
pub type OperandSize = u8;

#[derive(Debug)]
pub enum ImmediateValue {
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

/// Check if reg is a 32 bit register
#[rustfmt::skip]
pub fn is_32bit_reg(reg: &Register) -> bool {
    match reg {
        EAX  | EBX  | ECX  | EDX  | ESI | EDI | ESP | EBP | R8D | R9D | R10D |
        R11D | R12D | R13D | R14D | R15D => true,
        _ => false,
    }
}
