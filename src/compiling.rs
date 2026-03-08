use crate::abstract_syntax_tree;
use crate::abstract_syntax_tree::{
    Expression, Function, Operator, Program, Statement, Type, Variable,
};
use crate::assembling::ImmediateValue;
use crate::assembling::{
    ImmediateValue::*,
    Instruction::{self, *},
    Operand::*,
    Register::*,
};
use crate::linking::elf::SegmentType;
use std::collections::HashMap;

/// Struct containing the raw bytecode and data, still needs to be converted to elf/linked
pub struct CompilationResult {
    pub code: Vec<Instruction>,
    pub data: HashMap<crate::assembling::Label, Vec<u8>>,
}

const MAIN_LABEL: &str = "main";

/// Generates bytecode section, and string section from AST
pub fn compile(program: Program) -> CompilationResult {
    let mut output = CompilationResult {
        code: vec![],
        data: HashMap::new(),
    };
    compile_program(program, &mut output);
    output
}

fn compile_program(program: Program, output: &mut CompilationResult) {
    let Program {
        functions,
        variables,
    } = program;

    output.code.append(&mut vec![
        // Call the main function
        Call(Immediate(Label(MAIN_LABEL.to_string(), SegmentType::Text))),
        //	Move return value of main into exit code argument
        Mov(Register(RDI), Register(RAX)),
        //	sys_exit system call
        Mov(Register(RAX), Immediate(Literal(0x3c))),
        //	syscall
        Syscall,
    ]);

    for function in functions {
        compile_function(function, output);
    }

    for variable in variables {
        compile_variable(variable, output);
    }
}

fn format_return_label(function_name: &str) -> String {
    format!("return+{function_name}")
}

fn compile_variable(_variable: Variable, _output: &mut CompilationResult) {
    todo!()
}

fn compile_function(function: Function, output: &mut CompilationResult) {
    if function.arguments.len() > 0 {
        todo!("function arguments not supported yet")
    }

    output.code.append(&mut vec![
        ILabel(function.indentifier.clone()),
        PushR(RBP),
        Mov(Register(RBP), Register(RSP)),
        // TODO: adjust RSP for local variables
    ]);

    for statement in function.body.iter() {
        compile_statement(statement.clone(), &function, output);
    }

    output.code.append(&mut vec![
        ILabel(format_return_label(&function.indentifier)),
        // TODO: adjust rsp again, to free the local variables
        Leave,
        Ret,
    ]);
}

fn compile_statement(
    statement: Statement,
    current_function: &Function,
    output: &mut CompilationResult,
) {
    match statement {
        Statement::Declaration(variable) => todo!(),
        Statement::Expression(expression) => {
            compile_expression(expression, output);
            // TODO: depends on type of expression
            output
                .code
                .push(Sub(Register(RSP), Immediate(Literal(0x8))));
        }
        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => todo!(),
        Statement::While { condition, body } => todo!(),
        Statement::Return(expression) => {
            compile_expression(expression, output);
            output.code.append(&mut vec![
                // Put expression result into RAX register
                Pop(Register(RAX)),
                // Jump to return code of function
                Jmp(Immediate(ImmediateValue::Label(
                    format_return_label(&current_function.indentifier),
                    SegmentType::Text,
                ))),
            ]);
        }
    }
}

fn compile_expression(expression: Expression, output: &mut CompilationResult) {
    match expression {
        Expression::Literal(literal) => compile_literal(literal, output),
        Expression::Var(_) => todo!(),
        Expression::Operator(operator, expression, expression1) => todo!(),
        Expression::FunctionCall(_, expressions) => todo!(),
    }
}

fn compile_literal(literal: abstract_syntax_tree::Literal, output: &mut CompilationResult) {
    match literal {
        abstract_syntax_tree::Literal::Bool(b) => {
            let val = if b { 1 } else { 0 };
            output.code.push(PushI(Literal(val), 1))
        }
        abstract_syntax_tree::Literal::Int(i) => {
            // No push instruction with immediate 8 byte size, so we divide over two operations
            // TODO: order right here?
            // output.code.push(PushI(Literal(i & 0xFFFFFFFF), 4));
            // output.code.push(PushI(Literal(i >> 32), 4))
            // TODO: Seems like push and pop are always 64 bit?
            // https://stackoverflow.com/questions/43435764/64-bit-mode-does-not-support-32-bit-push-and-pop-instructions
            output.code.push(PushI(Literal(i), 4))
        }
        abstract_syntax_tree::Literal::String(_) => todo!(),
    }
}
