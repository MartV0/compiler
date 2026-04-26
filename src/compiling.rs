mod compile_expression;

use crate::abstract_syntax_tree;
use crate::abstract_syntax_tree::{Expression, Function, Program, Statement, Variable};
use crate::assembling::assembly::ImmediateValue;
use crate::assembling::assembly::{
    ImmediateValue::*,
    Instruction::{self, *},
    Operand::*,
    Register::*,
};
use crate::linking::elf::SegmentType;
use compile_expression::{ExpressionResult::*, compile_expression};
use rand::distr::{Alphanumeric, SampleString};
use std::collections::HashMap;

/// Struct containing the raw bytecode and data, still needs to be converted to elf/linked
pub struct CompilationResult {
    pub code: Vec<Instruction>,
    pub data: HashMap<crate::assembling::assembly::Label, Vec<u8>>,
}

struct Environment<'a> {
    // Local variables, map from variable name to offset relative to rbp
    local: HashMap<&'a str, i32>,
}

/// name used for main function
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

/// Compiles the program
/// Outputs code for every variable and function, and code that calls the main function
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

    for variable in variables {
        compile_variable(variable, output);
    }

    for function in functions {
        compile_function(function, output);
    }
}

/// Create label for return section
fn format_return_label(function_name: &str) -> String {
    format!("return+{function_name}")
}

/// Create random label
fn format_random_label(str: &str) -> String {
    let mut label = str.to_string();
    let mut rng = rand::rng();
    Alphanumeric.append_string(&mut rng, &mut label, 20);
    label
}

/// Create label for variable section
fn format_variable_label(identifier: &str) -> String {
    format!("globalvar:{identifier}")
}

/// Compile a global variable declaration
fn compile_variable(variable: Variable, output: &mut CompilationResult) {
    // TODO: depends on size of type
    output
        .data
        .insert(format_variable_label(&variable.identifier), [0; 8].to_vec());
}

/// Compile a function definition
fn compile_function(function: Function, output: &mut CompilationResult) {
    let mut env = Environment {
        local: HashMap::new(),
    };
    //rbp=previous saved rbp
    //rbp+8=return adress
    //rbp+16=last func arg
    //rbp+24=second to last func arg
    let mut offset: i32 = 16;
    for arg in function.arguments.iter().rev() {
        env.local.insert(&arg.identifier, offset);
        // TODO: depends on arg size
        offset += 8;
    }

    output.code.append(&mut vec![
        ILabel(function.indentifier.clone()),
        Push(Register(RBP)),
        Mov(Register(RBP), Register(RSP)),
        // TODO: adjust RSP for local variables
    ]);

    compile_block(&function.body, &function, output, &mut env);

    output.code.append(&mut vec![
        ILabel(format_return_label(&function.indentifier)),
        // TODO: adjust rsp again, to free the local variables
        Leave,
        Ret,
    ]);
}

/// Compile a statement
fn compile_statement(
    statement: Statement,
    current_function: &Function,
    output: &mut CompilationResult,
    env: &mut Environment,
) {
    match statement {
        Statement::Declaration(_) => todo!(),
        Statement::Expression(expression) => {
            compile_expression(expression, output, env, Value);
            // TODO: depends on type of expression
            // Expression left result on the stack, pop this
            output
                .code
                .push(Add(Register(RSP), Immediate(Literal(0x8))));
        }
        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => {
            compile_if_statement(
                condition,
                then_branch,
                else_branch,
                current_function,
                output,
                env,
            );
        }
        Statement::While { condition, body } => {
            compile_while_statement(condition, body, current_function, output, env)
        }
        Statement::Return(expression) => {
            compile_expression(expression, output, env, Value);
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

fn compile_if_statement(
    condition: Expression,
    then_branch: Vec<Statement>,
    else_branch: Vec<Statement>,
    current_function: &Function,
    output: &mut CompilationResult,
    env: &mut Environment,
) {
    // Compile condition
    compile_expression(condition, output, env, Value);
    let begin_else_label = format_random_label("begin_else+");
    output.code.append(&mut vec![
        // Put condition result into R14
        Pop(Register(R14)),
        // Cmp to zero
        Cmp(Register(R14), Immediate(Literal(0))),
        // Jump over then branch if condition is zero
        JE(ImmediateValue::Label(
            begin_else_label.clone(),
            SegmentType::Text,
        )),
    ]);

    // Compile then branch
    compile_block(&then_branch, current_function, output, env);

    let end_else_label = format_random_label("end_else+");
    output.code.append(&mut vec![
        // Jump over else branch
        Jmp(Immediate(ImmediateValue::Label(
            end_else_label.clone(),
            SegmentType::Text,
        ))),
        ILabel(begin_else_label),
    ]);

    // Compile else branch
    compile_block(&else_branch, current_function, output, env);

    output.code.push(ILabel(end_else_label));
}

fn compile_while_statement(
    condition: Expression,
    body: Vec<Statement>,
    current_function: &Function,
    output: &mut CompilationResult,
    env: &mut Environment,
) {
    let body_label = format_random_label("while_body+");
    let condition_label = format_random_label("while_condition+");
    output.code.append(&mut vec![
        // Jump to condition
        Jmp(Immediate(ImmediateValue::Label(
            condition_label.clone(),
            SegmentType::Text,
        ))),
        ILabel(body_label.clone()),
    ]);
    compile_block(&body, current_function, output, env);
    output.code.push(ILabel(condition_label));
    compile_expression(condition, output, env, Value);
    output.code.append(&mut vec![
        // Put condition result into R14
        Pop(Register(R14)),
        // Cmp to zero
        Cmp(Register(R14), Immediate(Literal(0))),
        // Jump to body if condition is not zero
        JNE(ImmediateValue::Label(body_label, SegmentType::Text)),
    ]);
}

fn compile_block(
    statements: &Vec<Statement>,
    current_function: &Function,
    output: &mut CompilationResult,
    env: &mut Environment,
) {
    for statement in statements {
        compile_statement(statement.clone(), &current_function, output, env);
    }
}
