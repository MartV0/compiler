// Functions to create the building blocks of instructions, such as mod/rm byte
use crate::assembling::assembly::*;
use crate::assembling::*;

pub enum RegValue {
    Register(Register),
    Extension(u8),
    None
}

/// Add opcode, modrm byte and optionally the rex byte infront
/// - output: where to output the bytecode
/// - rm: r/m argument, usually the first operand
/// - reg: reg argument, usually the second operand, or opcode extension
pub fn add_rex_opcode_modrm(
    output: &mut IntermediateAssemblingResult,
    mut opcode: Vec<u8>,
    rm: Register,
    reg: RegValue,
) {
    // Create 4 bit register codes
    let rm_bits = reg_to_XREG_bits(&rm);
    let reg_bits = match reg {
        RegValue::Register(reg) => reg_to_XREG_bits(&reg),
        RegValue::Extension(ext) => ext,
        RegValue::None => 0,
    };

    // Create rex prefix
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

    // Create modr/m byte
    // TODO: addressing modes
    let mod_ = 0b11;
    let reg = reg_bits & 0b111;
    let rm = rm_bits & 0b111;
    let mod_reg_rm = mod_ << 6 | reg << 3 | rm;
    output.code.push(mod_reg_rm);
}

/// Same as add_rex_opcode_modrm but assumes 64 bit size operants, meaning
/// rex.w byte is not needed. Some instructions default to 64 bit operands
pub fn add_rex_opcode_modrm64bit(
    output: &mut IntermediateAssemblingResult,
    opcode: Vec<u8>,
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

/// Adds the immediate part to the instruction
pub fn add_immediate(
    output: &mut IntermediateAssemblingResult,
    immediate: ImmediateValue,
    immediate_bytes: u8,
) {
    // immediate and offset follow basically the same logic
    add_offset(output, immediate, immediate_bytes, LabelType::Absolute);
}

/// Adds the offset part to the instruction
pub fn add_offset(
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
                // Add to code label positions to be processed afterwards,
                // all 0 bytes as a placeholder
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

/// Convert 64 bit register into 32 bit equivalent
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
