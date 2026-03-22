use crate::abstract_syntax_tree;
use crate::abstract_syntax_tree::Expression;
use crate::assembling::assembly::ImmediateValue;
use crate::assembling::assembly::{
    ImmediateValue::*,
    Instruction::{self, *},
    Operand::*,
    Register::*,
};
use crate::linking::elf::SegmentType;
use super::CompilationResult;

/// Compile an expression, leaves result of the expression on the stack
pub fn compile_expression(expression: Expression, output: &mut CompilationResult) {
    match expression {
        Expression::Literal(literal) => compile_literal(literal, output),
        Expression::Var(_) => todo!(),
        Expression::Operator(operator, expression, expression1) => todo!(),
        Expression::FunctionCall(_, expressions) => todo!(),
        Expression::BuiltInFunctionCall(name, expressions) => match name.as_str() {
            "syscall" => compile_syscall(expressions, output),
            x => todo!("{x}")
        },
    }
}

/// Compile a literal expression
fn compile_literal(literal: abstract_syntax_tree::Literal, output: &mut CompilationResult) {
    match literal {
        abstract_syntax_tree::Literal::Bool(b) => {
            let val = if b { 1 } else { 0 };
            output.code.push(Push(Immediate(Literal(val))))
        }
        abstract_syntax_tree::Literal::Int(i) => {
            // TODO: Seems like push and pop are always 64 bit?
            // https://stackoverflow.com/questions/43435764/64-bit-mode-does-not-support-32-bit-push-and-pop-instructions
            output.code.push(Push(Immediate(Literal(i))))
        }
        abstract_syntax_tree::Literal::String(str) => {
            let mut string_data = str.as_bytes().to_vec();
            // Null terminated string
            string_data.push(0);
            let label = format!("string_literal:{str}");
            output.data.insert(label.clone(), string_data);
            output.code.push(Push(Immediate(Label(label, SegmentType::Data))));
        },
    }
}

/// Compile systemcall expression, leaves 64 bit result from RAX register on the stack
fn compile_syscall(arguments: Vec<Expression>, output: &mut CompilationResult) {
    let len = arguments.len();
    if len < 1 {
        panic!("No syscall number provided")
    }
    if len > 7 {
        panic!("too much arguments to syscall (max 6)")
    }

    for expression in arguments.into_iter() {
        compile_expression(expression, output);
    }

    let pop_arguments = vec![
        Pop(Register(RAX)),
        Pop(Register(RDI)),
        Pop(Register(RSI)),
        Pop(Register(RDX)),
        Pop(Register(R10)),
        Pop(Register(R8)),
        Pop(Register(R9)),
    ];

    for x in pop_arguments[0..len].iter().rev() {
        output.code.push(x.clone());
    }

    output.code.append(&mut vec![
        Syscall,
        // Push result onto stack
        Push(Register(RAX))
    ]);
}
