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
