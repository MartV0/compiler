use super::CompilationResult;
use crate::abstract_syntax_tree as ast;
use crate::abstract_syntax_tree::Expression;
use crate::assembling::assembly::ImmediateValue;
use crate::assembling::assembly::{
    ImmediateValue::*,
    Instruction::{self, *},
    Operand::*,
    Register::*,
};
use crate::linking::elf::SegmentType;
use super::Environment;

/// Compile an expression, leaves result of the expression on the stack
pub fn compile_expression(expression: Expression, output: &mut CompilationResult, env: &mut Environment) {
    match expression {
        Expression::Literal(literal) => compile_literal(literal, output),
        Expression::Var(var) => compile_variable(var, output, env),
        Expression::Operator(operator, expression, expression1) => compile_operator(operator, *expression, *expression1, output, env),
        Expression::FunctionCall(identifier, arguments) => {
            compile_function_call(identifier, arguments, output, env);
        }
        Expression::BuiltInFunctionCall(name, expressions) => match name.as_str() {
            "syscall" => compile_syscall(expressions, output, env),
            x => todo!("{x}"),
        },
    }
}

/// Compile a literal expression
fn compile_literal(literal: ast::Literal, output: &mut CompilationResult) {
    match literal {
        ast::Literal::Bool(b) => {
            let val = if b { 1 } else { 0 };
            output.code.push(Push(Immediate(Literal(val))))
        }
        ast::Literal::Int(i) => {
            // TODO: Seems like push and pop are always 64 bit?
            // https://stackoverflow.com/questions/43435764/64-bit-mode-does-not-support-32-bit-push-and-pop-instructions
            output.code.push(Push(Immediate(Literal(i))))
        }
        ast::Literal::String(str) => {
            let mut string_data = str.as_bytes().to_vec();
            // Null terminated string
            string_data.push(0);
            let label = format!("string_literal:{str}");
            output.data.insert(label.clone(), string_data);
            output
                .code
                .push(Push(Immediate(Label(label, SegmentType::Data))));
        }
    }
}

/// Compile function call
/// Pushes arguments to stack, first arguments on the bottom
fn compile_function_call(
    identifier: ast::Indentifier,
    arguments: Vec<Expression>,
    output: &mut CompilationResult,
    env: &mut Environment
) {
    for expression in arguments.into_iter() {
        compile_expression(expression, output, env);
    }

    output.code.append(&mut vec![
        Call(Immediate(Label(identifier, SegmentType::Text))),
        // Push function result onto the stack
        Push(Register(RAX)),
    ]);
}

/// Compile systemcall expression, leaves 64 bit result from RAX register on the stack
fn compile_syscall(arguments: Vec<Expression>, output: &mut CompilationResult, env: &mut Environment) {
    
    let len = arguments.len();
    if len < 1 {
        panic!("No syscall number provided")
    }
    if len > 7 {
        panic!("too much arguments to syscall (max 6)")
    }

    for expression in arguments.into_iter() {
        compile_expression(expression, output, env);
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
        Push(Register(RAX)),
    ]);
}

/// Compile operator experession
fn compile_operator(
    operator: ast::Operator,
    operand1: Expression,
    operand2: Expression,
    output: &mut CompilationResult,
    env: &mut Environment
) {
    compile_expression(operand1, output, env);
    compile_expression(operand2, output, env);

    output.code.append(&mut vec![
        // Pop operand2 into R15
        Pop(Register(R15)),
        // Pop operand1 into R14
        Pop(Register(R14)),
    ]);
    
    output.code.push(match operator {
        ast::Operator::Addition => Add(Register(R14), Register(R15)),
        op => todo!("Operator not supported {op:?}")
    });

    output.code.push(Push(Register(R14)));
}

/// Compile variable expression
fn compile_variable(
    identifier: String,
    output: &mut CompilationResult,
    env: &mut Environment
) {
    let offset = env.local.get(identifier.as_str()).expect("Undefined variable");
    output.code.push(Push(IndirectOffset(RBP, *offset)));
}
