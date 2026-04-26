use super::CompilationResult;
use super::Environment;
use crate::abstract_syntax_tree::Expression;
use crate::abstract_syntax_tree::{self as ast, Operator};
use crate::assembling::assembly::{
    ImmediateValue::*,
    Instruction::{self, *},
    Operand::*,
    Register::*,
};
use crate::compiling::format_variable_label;
use crate::linking::elf::SegmentType;

// Whether the expression should result in a address or value
// example with assignment: a = b
// a should return an adress
// b should return an value
#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionResult {
    Value,
    Adress
}

/// Compile an expression, leaves result of the expression on the stack
pub fn compile_expression(expression: Expression, output: &mut CompilationResult, env: &mut Environment, result: ExpressionResult) {
    match expression {
        Expression::Literal(literal) => compile_literal(literal, output),
        Expression::Var(var) => compile_variable(var, output, env, result),
        Expression::Operator(operator, expression, expression1) => compile_operator(operator, *expression, *expression1, output, env, result),
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
        compile_expression(expression, output, env, ExpressionResult::Value);
    }

    output.code.append(&mut vec![
        Call(Immediate(Label(identifier, SegmentType::Text))),
        // Push function result onto the stack
        Push(Register(RAX)),
    ]);

    // TODO: shrink stack to free up arguments again?
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
        compile_expression(expression, output, env, ExpressionResult::Value);
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
    env: &mut Environment,
    result: ExpressionResult
) {
    let instructions = &mut match operator {
        Operator::Assignment => {
            compile_assignment(operand1, operand2, output, env, result);
            return;
        }
        Operator::Division => {
            compile_division(DivisionResult::Quotient, operand1, operand2, output, env, result);
            return;
        },
        Operator::Modulo => {
            compile_division(DivisionResult::Remainder, operand1, operand2, output, env, result);
            return;
        },
        Operator::Addition => vec![Add(Register(R14), Register(R15))],
        Operator::Subtraction => vec![Sub(Register(R14), Register(R15))],
        Operator::Multiplication => vec![IMul(Register(R14), Register(R15))],
        Operator::And => vec![And(Register(R14), Register(R15))],
        Operator::Or => vec![Or(Register(R14), Register(R15))],
        Operator::LessEq => vec![
            Cmp(Register(R14), Register(R15)),
            SetLE(Register(R14B))
        ],
        Operator::Less => vec![
            Cmp(Register(R14), Register(R15)),
            SetL(Register(R14B))
        ],
        Operator::GreaterEquals => vec![
            Cmp(Register(R14), Register(R15)),
            SetGE(Register(R14B))
        ],
        Operator::Greater => vec![
            Cmp(Register(R14), Register(R15)),
            SetG(Register(R14B))
        ],
        Operator::Equals => vec![
            Cmp(Register(R14), Register(R15)),
            SetE(Register(R14B))
        ],
        Operator::NotEqual => vec![
            Cmp(Register(R14), Register(R15)),
            SetNE(Register(R14B))
        ],
    };
    // TODO: result doorgeven?
    compile_expression(operand1, output, env, result.clone());
    compile_expression(operand2, output, env, result);

    output.code.append(&mut vec![
        // Pop operand2 into R15
        Pop(Register(R15)),
        // Pop operand1 into R14
        Pop(Register(R14)),
    ]);
    
    output.code.append(instructions);

    output.code.push(Push(Register(R14)));
}

enum DivisionResult {
    Quotient,
    Remainder
}

fn compile_division(
    div_result: DivisionResult,
    operand1: Expression,
    operand2: Expression,
    output: &mut CompilationResult,
    env: &mut Environment,
    result: ExpressionResult
) {
    // TODO: result doorgeven?
    compile_expression(operand1, output, env, result.clone());
    compile_expression(operand2, output, env, result);

    output.code.append(&mut vec![
        Pop(Register(R14)),
        Pop(Register(RAX)),
    ]);
    
    output.code.append(&mut vec![
        Mov(Register(RDX), Immediate(Literal(0))),
        IDiv(Register(R14)),
        Push(Register(match div_result {
            DivisionResult::Quotient => RAX,
            DivisionResult::Remainder => RDX,
        })),
    ]);
}

/// Compiles assignment operator
fn compile_assignment(
    operand1: Expression,
    operand2: Expression,
    output: &mut CompilationResult,
    env: &mut Environment,
    result: ExpressionResult
) {
    // Compile target address
    compile_expression(operand1, output, env, ExpressionResult::Adress);
    // Compile value
    // TODO: result doorgeven?
    compile_expression(operand2, output, env, result);

    output.code.append(&mut vec![
        // Pop value into R15
        Pop(Register(R15)),
        // Pop target address into R14
        Pop(Register(R14)),
        // Assign value
        Mov(Indirect(R14), Register(R15)),
        // Push value onto stack again
        Push(Register(R15))
    ]);
}

/// Compile variable expression
fn compile_variable(
    identifier: String,
    output: &mut CompilationResult,
    env: &mut Environment,
    result: ExpressionResult,
) {
    match env.local.get(identifier.as_str()) {
        Some(offset) => compile_local_variable(output, *offset, result),
        None => {
            let label = format_variable_label(&identifier);
            if output.data.contains_key(&label) {
                compile_global_variable(label, output, result);
            }
            else {
                panic!("Variable not found: {identifier}");
            }
        }
    }
}

/// compile local variable expression
fn compile_local_variable(
    output: &mut CompilationResult,
    offset: i32,
    result: ExpressionResult,
) {
    match result {
        ExpressionResult::Value => output.code.push(Push(IndirectDisplacement(RBP, offset))),
        ExpressionResult::Adress => output.code.append(&mut vec![
            LEA(Register(R14), IndirectDisplacement(RBP, offset)),
            Push(Register(R14)),
        ]),
    }
}

fn compile_global_variable(
    label: String,
    output: &mut CompilationResult,
    result: ExpressionResult,
) {
    match result {
        ExpressionResult::Value => output.code.append(&mut vec![
            Mov(Register(R15), Immediate(Label(label, SegmentType::Data))),
            Push(Indirect(R15))
        ]),
        ExpressionResult::Adress => output.code.push(Push(Immediate(Label(label, SegmentType::Data))))
    }
}
