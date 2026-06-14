use super::CompilationResult;
use super::Environment;
use crate::assembling::assembly::{
    ImmediateValue::*,
    Instruction::*,
    Operand::*,
    Register::*,
};
use crate::compiling::format_variable_label;
use crate::linking::elf::SegmentType;

use crate::abstract_syntax_tree::{self as ast, ExprType, Type, Operator};

pub type Expression = ast::Expression<ExprType>;

// Whether the expression should result in a address or value
// example with assignment: a = b
// a should return an address
// b should return an value
#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionResult {
    Value,
    Address,
}

/// Compile an expression, leaves result of the expression on the stack
pub fn compile_expression(
    expression: ExprType,
    output: &mut CompilationResult,
    env: &mut Environment,
    result: ExpressionResult,
) {
    let ExprType (expression, type_) = expression;
    match expression {
        Expression::Literal(literal) => compile_literal(literal, output),
        Expression::Var(var) => compile_variable(var, output, env, result),
        Expression::BinaryOp(operator, expression, expression1) => {
                compile_binary_operator(operator, *expression, *expression1, type_, output, env, result)
            }
        Expression::FunctionCall(identifier, arguments) => {
                compile_function_call(identifier, arguments, output, env);
            }
        Expression::BuiltInFunctionCall(name, expressions) => match name.as_str() {
                "syscall" => compile_syscall(expressions, output, env),
                x => panic!("Unsupported builtin function call: {x}"),
            },
        Expression::UnaryOp(unary_operator, expression) => compile_unary_operator(unary_operator, *expression, output, env, result),
    }
}

/// Compile a cast expression
fn compile_cast(_type: Type, operand: ExprType, output: &mut CompilationResult,
    env: &mut Environment,
    result: ExpressionResult,
    ) {
    compile_expression(operand, output, env, result);
    // all the supported casts don't do any logic, just for the type checker, so no code here
    // TODO: maybe I need to zero some bits when converting to char/bool from Int
}

/// Compile a literal expression
fn compile_literal(literal: ast::Literal, output: &mut CompilationResult) {
    match literal {
        ast::Literal::Bool(b) => {
            let val = if b { 1 } else { 0 };
            output.code.push(Push(Immediate(Literal(val))))
        }
        ast::Literal::Int(i) => {
            // Seems like push and pop are always 64 bit?
            // https://stackoverflow.com/questions/43435764/64-bit-mode-does-not-support-32-bit-push-and-pop-instructions
            // Use mov so we can actually use a 64 bit immediate
            output.code.push(Mov(Register(R15), Immediate(Literal(i))));
            output.code.push(Push(Register(R15)));
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
        ast::Literal::Char(char) => {
            output.code.push(Push(Immediate(Literal(char.into()))))
        }
    }
}

/// Compile function call
/// Pushes arguments to stack, first arguments on the bottom
fn compile_function_call(
    identifier: ast::Indentifier,
    arguments: Vec<ExprType>,
    output: &mut CompilationResult,
    env: &mut Environment,
) {
    let arg_size: i64 = arguments.iter().map(| arg | type_size_stack(&arg.1) as i64).sum();
    for expression in arguments.into_iter() {
        compile_expression(expression, output, env, ExpressionResult::Value);
    }

    output.code.append(&mut vec![
        Call(Immediate(Label(identifier, SegmentType::Text))),
        // Free up arguments again
        Add(Register(RSP), Immediate(Literal(arg_size))),
        // Push function result onto the stack
        Push(Register(RAX)),
    ]);
}

/// Compile systemcall expression, leaves 64 bit result from RAX register on the stack
fn compile_syscall(
    arguments: Vec<ExprType>,
    output: &mut CompilationResult,
    env: &mut Environment,
) {
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
fn compile_binary_operator(
    operator: ast::Operator,
    operand1: ExprType,
    operand2: ExprType,
    type_: Type,
    output: &mut CompilationResult,
    env: &mut Environment,
    result: ExpressionResult,
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
        Operator::ArraySubScript => {
            compile_array_subscript(operand1, operand2, type_, output, env, result);
            return;
        },
        Operator::Addition => vec![Add(Register(R14), Register(R15))],
        Operator::Subtraction => vec![Sub(Register(R14), Register(R15))],
        Operator::Multiplication => vec![IMul(Register(R14), Register(R15))],
        Operator::And => vec![And(Register(R14), Register(R15))],
        Operator::Or => vec![Or(Register(R14), Register(R15))],
        Operator::LessEq => vec![
            Cmp(Register(R14), Register(R15)),
            // TODO: if we just use 8 bit registers for all we wouldn't need this?
            // set entire register to 0, otherwise upper 7 bytes might still contain non-zero values
            Mov(Register(R14), Immediate(Literal(0))),
            SetLE(Register(R14B))
        ],
        Operator::Less => vec![
            Cmp(Register(R14), Register(R15)),
            Mov(Register(R14), Immediate(Literal(0))),
            SetL(Register(R14B))
        ],
        Operator::GreaterEquals => vec![
            Cmp(Register(R14), Register(R15)),
            Mov(Register(R14), Immediate(Literal(0))),
            SetGE(Register(R14B))
        ],
        Operator::Greater => vec![
            Cmp(Register(R14), Register(R15)),
            Mov(Register(R14), Immediate(Literal(0))),
            SetG(Register(R14B))
        ],
        Operator::Equals => vec![
            Cmp(Register(R14), Register(R15)),
            Mov(Register(R14), Immediate(Literal(0))),
            SetE(Register(R14B))
        ],
        Operator::NotEqual => vec![
            Cmp(Register(R14), Register(R15)),
            Mov(Register(R14), Immediate(Literal(0))),
            SetNE(Register(R14B))
        ],
    };
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
    Remainder,
}

fn compile_division(
    div_result: DivisionResult,
    operand1: ExprType,
    operand2: ExprType,
    output: &mut CompilationResult,
    env: &mut Environment,
    result: ExpressionResult,
) {
    compile_expression(operand1, output, env, result.clone());
    compile_expression(operand2, output, env, result);

    output
        .code
        .append(&mut vec![Pop(Register(R14)), Pop(Register(RAX))]);

    output.code.append(&mut vec![
        Mov(Register(RDX), Immediate(Literal(0))),
        IDiv(Register(R14)),
        Push(Register(match div_result {
            DivisionResult::Quotient => RAX,
            DivisionResult::Remainder => RDX,
        })),
    ]);
}

/// Compiles array subscript
fn compile_array_subscript(
    operand1: ExprType,
    operand2: ExprType,
    type_: Type,
    output: &mut CompilationResult,
    env: &mut Environment,
    result: ExpressionResult,
) {
    // Even though this value needs to return an address, it is already a pointer type, so the value
    // contains an address already
    compile_expression(operand1, output, env, ExpressionResult::Value);
    compile_expression(operand2, output, env, ExpressionResult::Value);
    output
        .code
        .append(&mut vec![
            // pop offset into r15
            Pop(Register(R15)),
            // pop base address into r14
            Pop(Register(R14)),
        ]);

    let size = type_size(&type_).into();
    // If operand size is not a byte, we need to multiply the offset by the operand size
    if size != 1 {
        output
            .code
            .append(&mut vec![
                Mov(Register(R13), Immediate(Literal(size))),
                IMul(Register(R15), Register(R13))
            ]);
    }

    // Add offset to base address
    output.code.push(Add(Register(R15), Register(R14)));


    match result {
        ExpressionResult::Value => {
            output
                .code
                .append(&mut match size {
                    8 => vec![Push(Indirect(R15))],
                    // If type only has a single byte we don't want to move 8 bytes from the address
                    // only 1, so we have to use movzx, which sets the other bytes to zero
                    1 => vec![
                        MovZX(Register(R15), Indirect(R15)),
                        Push(Register(R15))
                    ],
                    _ => panic!("Unsupported array size")
                })
        },
        ExpressionResult::Address => output.code.push(Push(Register(R15))),
    }
}

/// Returns the size of type in bytes
pub fn type_size(type_: &Type) -> u32 {
    match type_ {
        Type::Bool => 1,
        Type::Int => 8,
        Type::Void => 0,
        Type::Char => 1,
        Type::Pointer(_) => 8,
    }
}

/// Returns the size of type in bytes on the stack
/// Different than the sizes on the heap as x64 doesn't really support pushes
/// smaller then 64 bits
pub fn type_size_stack(type_: &Type) -> u32 {
    match type_ {
        Type::Bool => 8,
        Type::Int => 8,
        Type::Void => 0,
        Type::Char => 8,
        Type::Pointer(_) => 8,
    }
}

/// Compiles assignment operator
fn compile_assignment(
    operand1: ExprType,
    operand2: ExprType,
    output: &mut CompilationResult,
    env: &mut Environment,
    result: ExpressionResult,
) {
    let size = type_size(&operand1.1);
    // Compile target address
    compile_expression(operand1, output, env, ExpressionResult::Address);
    // Compile value
    compile_expression(operand2, output, env, result);

    output.code.append(&mut vec![
        // Pop value into R15
        Pop(Register(R15)),
        // Pop target address into R14
        Pop(Register(R14)),
        // Assign value
        match size {
            8 => Mov(Indirect(R14), Register(R15)),
            1 => Mov(Indirect(R14), Register(R15B)),
            _ => panic!("unsupported assignment size")
        },
        // Push value onto stack again
        Push(Register(R15)),
    ]);
}

/// Compile operator experession
fn compile_unary_operator(
    operator: ast::UnaryOperator,
    operand: ExprType,
    output: &mut CompilationResult,
    env: &mut Environment,
    result: ExpressionResult,
) {
    match operator {
        ast::UnaryOperator::Dereference => {
            match result {
                ExpressionResult::Value => {
                    compile_expression(operand, output, env, result);
                    output.code.append(&mut vec![
                        // Pop operand into R14
                        Pop(Register(R14)),
                        Push(Indirect(R14))
                    ]);
                },
                ExpressionResult::Address => {
                    // We need an address as result this negates the dereference in a way, so just
                    // call the next function but with value as result type
                    compile_expression(operand, output, env, ExpressionResult::Value);
                },
            }
        },
        ast::UnaryOperator::AddressOf => {
            match result {
                ExpressionResult::Value => {
                    compile_expression(operand, output, env, ExpressionResult::Address);
                },
                ExpressionResult::Address => {
                    panic!("Can't take address of address")
                },
            }
        },
        ast::UnaryOperator::Negation => {
            compile_expression(operand, output, env, result);
            output.code.append(&mut vec![
                Pop(Register(R14)),
                // Can't use bitwise not, as it flips all bits, not just the least significant one
                Xor(Register(R14), Immediate(Literal(1))),
                Push(Register(R14)),
            ]);
        },
        ast::UnaryOperator::Cast(type_) => compile_cast(type_, operand, output, env, result),
    };
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
            } else {
                panic!("Variable not found: {identifier}");
            }
        }
    }
}

/// compile local variable expression
fn compile_local_variable(output: &mut CompilationResult, offset: i32, result: ExpressionResult) {
    match result {
        ExpressionResult::Value => output.code.push(Push(IndirectDisplacement(RBP, offset))),
        ExpressionResult::Address => output.code.append(&mut vec![
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
            Push(Indirect(R15)),
        ]),
        ExpressionResult::Address => output
            .code
            .push(Push(Immediate(Label(label, SegmentType::Data)))),
    }
}
