use self::TypeError::*;
use std::{collections::HashMap, iter};

use crate::abstract_syntax_tree::{self, Expr, Type::{self, *}, Variable, Operator, UnaryOperator, Literal, map_to_expr, map_from_expr};

pub type Program = abstract_syntax_tree::Program<Expr>;
pub type Function = abstract_syntax_tree::Function<Expr>;
pub type Statement = abstract_syntax_tree::Statement<Expr>;
pub type Expression = abstract_syntax_tree::Expression<Expr>;

#[derive(Debug)]
pub enum TypeError {
    DuplicateVariable(String),
    UndefinedVariable(String),
    UndefinedFunction(String),
    UndefinedBuiltinFunction(String),
    WrongArgumentAmount(String),
    // Actual, expected
    WrongArgument(Type, Type, String),
    WrongReturn(Type),
    WrongCondition(Type),
    WrongOperand(Type, Type, Operator),
    WrongUnaryOperand(Type, UnaryOperator),
}

pub fn type_check(
    Program {
        functions,
        variables,
    }: &Program,
) -> Result<(), TypeError> {
    let mut defined_functions = HashMap::new();
    for Function {
        return_type,
        arguments,
        indentifier,
        ..
    } in functions.iter()
    {
        defined_functions.insert(
            indentifier.clone(),
            (return_type.clone(), arguments.clone()),
        );
    }

    let mut defined_variables = HashMap::new();
    for Variable { type_, identifier } in variables.iter() {
        defined_variables.insert(identifier.clone(), type_.clone());
    }

    for function in functions.into_iter() {
        check_function(function, &mut defined_variables, &defined_functions)?;
    }
    Ok(())
}

fn check_function(
    Function {
        return_type,
        arguments,
        indentifier,
        body,
    }: &Function,
    variables: &mut HashMap<String, Type>,
    functions: &HashMap<String, (Type, Vec<Variable>)>,
) -> Result<(), TypeError> {
    let mut defined_variables = variables.clone();
    for Variable { type_, identifier } in arguments {
        if let Some(_) = defined_variables.insert(identifier.clone(), type_.clone()) {
            return Err(DuplicateVariable(identifier.clone()));
        }
    }
    for statement in body {
        check_statement(statement.clone(), &return_type, &mut defined_variables, functions)?;
    }
    Ok(())
}

fn check_block(
    block: Vec<Statement>,
    return_type: &Type,
    variables: &HashMap<String, Type>,
    functions: &HashMap<String, (Type, Vec<Variable>)>
) -> Result<(), TypeError> {
    // clone variables as the variables defined in this block are local to it
    let mut variables = variables.clone();
    for statement in block {
        check_statement(statement, return_type, &mut variables, functions)?;
    }
    Ok(())
}

fn check_statement(
    statement: Statement,
    return_type: &Type,
    variables: &mut HashMap<String, Type>,
    functions: &HashMap<String, (Type, Vec<Variable>)>,
) -> Result<(), TypeError> {
    match statement {
        Statement::Declaration(Variable { type_, identifier }) => {
            match variables.insert(identifier.clone(), type_) {
                Some(_) => Err(DuplicateVariable(identifier)),
                None => Ok(()),
            }
        }
        Statement::Expression(expression) => check_expression(expression.0, variables, functions).map(| _ | ()),
        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let condition = check_expression(condition.0, variables, functions)?;
            if condition != Bool {
                Err(WrongCondition(condition))
            } else {
                check_block(then_branch, return_type, variables, functions)?;
                check_block(else_branch, return_type, variables, functions)?;
                Ok(())
            }
        },
        Statement::While { condition, body } => {
            let condition = check_expression(condition.0, variables, functions)?;
            if condition != Bool {
                Err(WrongCondition(condition))
            } else {
                check_block(body, return_type, variables, functions)?;
                Ok(())
            }
        },
        Statement::Return(expression) => {
            let actual_return = check_expression(expression.0, variables, functions)?;
            if *return_type != actual_return {
                Err(WrongReturn(actual_return))
            } else {
                Ok(())
            }

        },
    }
}

fn check_expression(
    expression: Expression,
    variables: &mut HashMap<String, Type>,
    functions: &HashMap<String, (Type, Vec<Variable>)>,
) -> Result<Type, TypeError> {
    match expression {
        Expression::Literal(Literal::Bool(_)) => Ok(Bool),
        Expression::Literal(Literal::Int(_)) => Ok(Int),
        Expression::Literal(Literal::String(_)) => Ok(Pointer(Box::new(Char))),
        Expression::Var(identfier) => match variables.get(&identfier) {
            Some(type_) => Ok(type_.clone()),
            None => Err(UndefinedVariable(identfier)),
        },
        Expression::BinaryOp(operator, operand1, operand2) => {
            check_binary_operator(operator, (*operand1).0, (*operand2).0, variables, functions)
        }
        Expression::UnaryOp(operator, operand) => {
            check_unary_operator(operator, (*operand).0, variables, functions)
        }
        Expression::FunctionCall(id, args) => check_function_call(id, map_from_expr(args), variables, functions),
        Expression::BuiltInFunctionCall(identifier, arguments) => check_builtinfunction_call(identifier, map_from_expr(arguments), variables, functions),
    }
}

fn check_builtinfunction_call(
    identifier: String,
    arguments: Vec<Expression>,
    variables: &mut HashMap<String, Type>,
    functions: &HashMap<String, (Type, Vec<Variable>)>,
) -> Result<Type, TypeError> {
    if identifier != "syscall" {
        return Err(UndefinedBuiltinFunction(identifier));
    }
    let mut arg_types = vec![];
    for argument in arguments.iter() {
        arg_types.push(check_expression(argument.clone(), variables, functions)?);
    }
    if arg_types.len() < 1 && arg_types.len() > 7 {
        return Err(WrongArgumentAmount(identifier));
    }
    // Can't really check other argument types, as they very wildly based on the syscall
    if arg_types[0] != Int {
        return Err(WrongArgument(arg_types[0].clone(), Int, identifier));
    }
    Ok(Int)
}

fn check_function_call(
    function_identifier: String,
    arguments: Vec<Expression>,
    variables: &mut HashMap<String, Type>,
    functions: &HashMap<String, (Type, Vec<Variable>)>,
) -> Result<Type, TypeError> {
    let mut arg_types = vec![];
    for argument in arguments.iter() {
        arg_types.push(check_expression(argument.clone(), variables, functions)?);
    }
    let (return_type, func_args) = match functions.get(&function_identifier) {
        Some(x) => x,
        None => return Err(UndefinedFunction(function_identifier)),
    };
    if arg_types.len() != func_args.len() {
        return Err(WrongArgumentAmount(function_identifier));
    }
    for (actual, Variable { type_: expected, .. }) in iter::zip(arg_types.into_iter(), func_args.into_iter()) {
        let expected = expected.clone();
        if actual != expected {
            return Err(WrongArgument(actual, expected, function_identifier));
        }
    }
    Ok(return_type.clone())
}

fn check_unary_operator(
    operator: UnaryOperator,
    operand: Expression,
    variables: &mut HashMap<String, Type>,
    functions: &HashMap<String, (Type, Vec<Variable>)>
) -> Result<Type, TypeError> {
    let operator_type = check_expression(operand, variables, functions)?;
    use UnaryOperator::*;
    match operator {
        Dereference => match operator_type {
            Pointer(type_) => Ok(*type_),
            _ => Err(WrongUnaryOperand(operator_type, operator)),
        },
        AddressOf => Ok(Pointer(Box::new(operator_type))),
        Negation => match operator_type {
            Bool => Ok(Bool),
            _ => Err(WrongUnaryOperand(operator_type, operator)),
        },
    }
}

fn check_binary_operator(
    operator: Operator,
    operand1: Expression,
    operand2: Expression,
    variables: &mut HashMap<String, Type>,
    functions: &HashMap<String, (Type, Vec<Variable>)>
) -> Result<Type, TypeError> {
    use Operator::*;
    let op1_type = check_expression(operand1, variables, functions)?;
    let op2_type = check_expression(operand2, variables, functions)?;
    match operator {
        Addition | Subtraction | Multiplication | Division | Modulo => match (op1_type, op2_type) {
            (Int, Int) => Ok(Int),
            (op1_type, op2_type) => Err(WrongOperand(op1_type, op2_type, operator)),
        },
        And | Or => match (op1_type, op2_type) {
            (Bool, Bool) => Ok(Bool),
            (op1_type, op2_type) => Err(WrongOperand(op1_type, op2_type, operator)),
        },
        LessEq | Less | GreaterEquals | Greater => match (op1_type, op2_type) {
            (Int, Int) => Ok(Bool),
            (op1_type, op2_type) => Err(WrongOperand(op1_type, op2_type, operator)),
        },
        Equals | NotEqual => {
            if op1_type != op2_type {
                Err(WrongOperand(op1_type, op2_type, operator))
            } else {
                Ok(Bool)
            }
        }
        Assignment => {
            // TODO: this should be probably also check if operand1 is some variable, not just some
            // constant or something for example
            if op1_type != op2_type {
                Err(WrongOperand(op1_type, op2_type, operator))
            } else {
                Ok(op2_type)
            }
        }
        ArraySubScript => match (op1_type, op2_type) {
            (Pointer(type_), Int) => Ok(*type_),
            (op1_type, op2_type) => Err(WrongOperand(op1_type, op2_type, operator)),
        },
    }
}
