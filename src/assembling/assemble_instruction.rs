use super::assembly::*;
use super::assemble_instruction_part::*;
use super::*;

/// Output the assembly of a single instruction
pub fn assemble_instruction(instruction: Instruction, output: &mut IntermediateAssemblingResult) {
    match instruction {
        Instruction::ILabel(label) => {
            output
                .code_labels
                .insert(label, output.code.len().try_into().unwrap());
        }
        Instruction::Mov(Operand::Register(reg), Operand::Immediate(val))
            if is_32bit_reg(&reg) | is_64bit_reg(&reg) =>
        {
            add_rex_opcode_modrm_offset(output, vec![0xC7], Operand::Register(reg), RegValue::None);
            add_immediate(output, val, 4);
        }
        Instruction::Mov(rm, Operand::Register(reg)) => {
            add_rex_opcode_modrm_offset(output, vec![0x89], rm, RegValue::Register(reg));
        }
        Instruction::Syscall => output.code.append(&mut vec![0x0F, 0x05]),
        Instruction::Leave => output.code.push(0xC9),
        Instruction::Ret => output.code.push(0xC3),
        Instruction::Call(Operand::Immediate(immediate)) => {
            output.code.push(0xE8);
            add_offset(
                output,
                immediate,
                4,
                LabelType::Relative
            );
        }
        Instruction::Jmp(Operand::Immediate(immediate)) => {
            output.code.push(0xE9);
            add_offset(
                output,
                immediate,
                4,
                LabelType::Relative,
            );
        }
        Instruction::Pop(Operand::Register(reg)) if is_64bit_reg(&reg) => {
            add_rex_opcode_modrm_offset(output, vec![0x8F], Operand::Register(reg), RegValue::None);
        }
        Instruction::Push(Operand::Immediate(immediate)) => {
            output.code.push(0x68);
            add_immediate(output, immediate, 4);
        }
        Instruction::Push(reg) => {
            add_rex_opcode_modrm64bit(output, vec![0xFF], reg, RegValue::Extension(0x06));
        }
        Instruction::Sub(Operand::Register(reg), Operand::Immediate(val))
            if is_32bit_reg(&reg) | is_64bit_reg(&reg) =>
        {
            add_rex_opcode_modrm_offset(output, vec![0x81], Operand::Register(reg), RegValue::Extension(5));
            add_immediate(output, val, 4);
        }
        Instruction::Add(Operand::Register(reg), Operand::Register(rm))
            if is_32bit_reg(&reg) | is_64bit_reg(&reg) =>
        {
            add_rex_opcode_modrm_offset(output, vec![0x03], Operand::Register(rm), RegValue::Register(reg));
        }
        val => panic!("unsupported instruction: {val:?}"),
    }
}
