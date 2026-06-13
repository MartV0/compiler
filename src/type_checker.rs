use self::TypeError::*;
use std::{collections::HashMap, iter};

use crate::abstract_syntax_tree::{*, Type::{self, *}};

#[derive(Debug)]
#[allow(dead_code)]
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
    WrongCast(Type, Type),
}

pub fn type_check(
    Program {
        functions,
        variables,
    }: Program<Expr>,
) -> Result<Program<ExprType>, TypeError> {
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

    let mut functions2 = vec![];
    for function in functions.into_iter() {
        functions2.push(check_function(function, &mut defined_variables, &defined_functions)?);
    }
    Ok(Program {
        variables,
        functions: functions2
    })
}

fn check_function(
    Function {
        return_type,
        arguments,
        indentifier,
        body,
    }: Function<Expr>,
    variables: &mut HashMap<String, Type>,
    functions: &HashMap<String, (Type, Vec<Variable>)>,
) -> Result<Function<ExprType>, TypeError> {
    let mut defined_variables = variables.clone();
    for Variable { type_, identifier } in arguments.iter() {
        if let Some(_) = defined_variables.insert(identifier.clone(), type_.clone()) {
            return Err(DuplicateVariable(identifier.clone()));
        }
    }
    let mut annotated_body = vec![];
    for statement in body {
        annotated_body.push(check_statement(statement.clone(), &return_type, &mut defined_variables, functions)?);
    }
    Ok(Function {
        return_type,
        arguments,
        indentifier,
        body: annotated_body
    })
}

fn check_block(
    block: Vec<Statement<Expr>>,
    return_type: &Type,
    variables: &HashMap<String, Type>,
    functions: &HashMap<String, (Type, Vec<Variable>)>
) -> Result<Vec<Statement<ExprType>>, TypeError> {
    // clone variables as the variables defined in this block are local to it
    let mut variables = variables.clone();
    let mut annoted_statements = vec![];
    for statement in block {
        annoted_statements.push(check_statement(statement, return_type, &mut variables, functions)?);
    }
    Ok(annoted_statements)
}

fn check_statement(
    statement: Statement<Expr>,
    return_type: &Type,
    variables: &mut HashMap<String, Type>,
    functions: &HashMap<String, (Type, Vec<Variable>)>,
) -> Result<Statement<ExprType>, TypeError> {
    use Statement::*;

    match statement {
        Declaration(Variable { type_, identifier }) => {
            match variables.insert(identifier.clone(), type_.clone()) {
                Some(_) => Err(DuplicateVariable(identifier)),
                None => Ok(Declaration(Variable { type_, identifier })),
            }
        }
        Expression(expression) => check_expression(expression.0, variables, functions).map(| expr | Expression(expr)),
        If {
            condition,
            then_branch,
            else_branch,
        } => {
            let condition = check_expression(condition.0, variables, functions)?;
            if condition.1 != Bool {
                Err(WrongCondition(condition.1))
            } else {
                Ok(If { 
                    condition: condition,
                    then_branch: check_block(then_branch, return_type, variables, functions)?,
                    else_branch: check_block(else_branch, return_type, variables, functions)?
                })
            }
        },
        While { condition, body } => {
            let condition = check_expression(condition.0, variables, functions)?;
            if condition.1 != Bool {
                Err(WrongCondition(condition.1))
            } else {
                Ok(While {
                    condition,
                    body: check_block(body, return_type, variables, functions)?
                })
            }
        },
        Return(expression) => {
            let actual_return = check_expression(expression.0, variables, functions)?;
            if *return_type != actual_return.1 {
                Err(WrongReturn(actual_return.1))
            } else {
                Ok(Return(actual_return))
            }

        },
    }
}

fn check_expression(
    expression: Expression<Expr>,
    variables: &mut HashMap<String, Type>,
    functions: &HashMap<String, (Type, Vec<Variable>)>,
) -> Result<ExprType, TypeError> {
    use crate::abstract_syntax_tree::Literal::*;
    use Expression::*;

    match expression {
        Literal(Bool(b)) => Ok(ExprType(Literal(Bool(b)), Type::Bool)),
        Literal(Int(i)) => Ok(ExprType(Literal(Int(i)), Type::Int)),
        Literal(String(s)) => Ok(ExprType(Literal(String(s)), Type::Pointer(Box::new(Type::Char)))),
        Var(identfier) => match variables.get(&identfier) {
            Some(type_) => Ok(ExprType(Var(identfier), type_.clone())),
            None => Err(UndefinedVariable(identfier)),
        },
        BinaryOp(operator, operand1, operand2) => {
            check_binary_operator(operator, (*operand1).0, (*operand2).0, variables, functions)
        }
        UnaryOp(operator, operand) => {
            check_unary_operator(operator, (*operand).0, variables, functions)
        }
        FunctionCall(id, args) => check_function_call(id, map_from_expr(args), variables, functions),
        BuiltInFunctionCall(identifier, arguments) => check_builtinfunction_call(identifier, map_from_expr(arguments), variables, functions),
        Cast(type_, operand) => check_cast(type_, *operand, variables, functions),
    }
}

fn check_cast(type_: Type, operand: Expr, variables: &mut HashMap<String, Type>, functions: &HashMap<String, (Type, Vec<Variable>)>) -> Result<ExprType, TypeError> {
    let operand = check_expression(operand.0, variables, functions)?;
    match (type_.clone(), operand.1.clone()) {
        (Int, Bool) |
        (Char, Bool) |
        (Int, Char)
            => Ok(ExprType(Expression::Cast(type_.clone(), Box::new(operand)), type_)),
        (cast_type, operand_type) => Err(WrongCast(cast_type, operand_type))
    }
}

fn check_builtinfunction_call(
    identifier: String,
    arguments: Vec<Expression<Expr>>,
    variables: &mut HashMap<String, Type>,
    functions: &HashMap<String, (Type, Vec<Variable>)>,
) -> Result<ExprType, TypeError> {
    if identifier != "syscall" {
        return Err(UndefinedBuiltinFunction(identifier));
    }
    let mut annotated_args = vec![];
    for argument in arguments.iter() {
        annotated_args.push(check_expression(argument.clone(), variables, functions)?);
    }
    if annotated_args.len() < 1 && annotated_args.len() > 7 {
        return Err(WrongArgumentAmount(identifier));
    }
    // Can't really check other argument types, as they vary wildly based on the syscall
    if annotated_args[0].1 != Int {
        return Err(WrongArgument(annotated_args[0].1.clone(), Int, identifier));
    }
    Ok(ExprType(Expression::BuiltInFunctionCall(identifier, annotated_args), Int))
}

fn check_function_call(
    function_identifier: String,
    arguments: Vec<Expression<Expr>>,
    variables: &mut HashMap<String, Type>,
    functions: &HashMap<String, (Type, Vec<Variable>)>,
) -> Result<ExprType, TypeError> {
    let mut annotated_args = vec![];
    for argument in arguments.iter() {
        annotated_args.push(check_expression(argument.clone(), variables, functions)?);
    }
    let (return_type, func_args) = match functions.get(&function_identifier) {
        Some(x) => x,
        None => return Err(UndefinedFunction(function_identifier)),
    };
    if annotated_args.len() != func_args.len() {
        return Err(WrongArgumentAmount(function_identifier));
    }
    for (actual, Variable { type_: expected, .. }) in iter::zip(annotated_args.iter(), func_args.iter()) {
        let expected = expected.clone();
        if actual.1 != expected {
            return Err(WrongArgument(actual.1.clone(), expected, function_identifier));
        }
    }
    Ok(ExprType(Expression::FunctionCall(function_identifier, annotated_args), return_type.clone()))
}

fn check_unary_operator(
    operator: UnaryOperator,
    operand: Expression<Expr>,
    variables: &mut HashMap<String, Type>,
    functions: &HashMap<String, (Type, Vec<Variable>)>
) -> Result<ExprType, TypeError> {
    let operator_type = check_expression(operand.clone(), variables, functions)?;
    use UnaryOperator::*;
    let expr = Expression::UnaryOp(operator.clone(), Box::new(operator_type.clone()));
    match operator {
        Dereference => match operator_type.1 {
            Pointer(type_) => Ok(ExprType(expr, *type_)),
            _ => Err(WrongUnaryOperand(operator_type.1, operator)),
        },
        AddressOf => Ok(ExprType(expr, Pointer(Box::new(operator_type.1)))),
        Negation => match operator_type.1 {
            Bool => Ok(ExprType(expr, Bool)),
            t => Err(WrongUnaryOperand(t, operator)),
        },
    }
}

fn check_binary_operator(
    operator: Operator,
    operand1: Expression<Expr>,
    operand2: Expression<Expr>,
    variables: &mut HashMap<String, Type>,
    functions: &HashMap<String, (Type, Vec<Variable>)>
) -> Result<ExprType, TypeError> {
    use Operator::*;
    let op1_type = check_expression(operand1, variables, functions)?;
    let op2_type = check_expression(operand2, variables, functions)?;

    let expr = Expression::BinaryOp(operator.clone(), Box::new(op1_type.clone()), Box::new(op2_type.clone()));
    match operator {
        Addition | Subtraction | Multiplication | Division | Modulo => match (op1_type.1, op2_type.1) {
            (Int, Int) => Ok(ExprType(expr, Int)),
            (op1_type, op2_type) => Err(WrongOperand(op1_type, op2_type, operator)),
        },
        And | Or => match (op1_type.1, op2_type.1) {
            (Bool, Bool) => Ok(ExprType(expr, Bool)),
            (op1_type, op2_type) => Err(WrongOperand(op1_type, op2_type, operator)),
        },
        LessEq | Less | GreaterEquals | Greater => match (op1_type.1, op2_type.1) {
            (Int, Int) => Ok(ExprType(expr, Bool)),
            (op1_type, op2_type) => Err(WrongOperand(op1_type, op2_type, operator)),
        },
        Equals | NotEqual => {
            if op1_type.1 != op2_type.1 {
                Err(WrongOperand(op1_type.1, op2_type.1, operator))
            } else {
                Ok(ExprType(expr, Bool))
            }
        }
        Assignment => {
            // TODO: this should be probably also check if operand1 is some variable, not just some
            // constant or something for example
            if op1_type.1 != op2_type.1 {
                Err(WrongOperand(op1_type.1, op2_type.1, operator))
            } else {
                Ok(ExprType(expr, op2_type.1))
            }
        }
        ArraySubScript => match (op1_type.1, op2_type.1) {
            (Pointer(type_), Int) => Ok(ExprType(expr, *type_)),
            (op1_type, op2_type) => Err(WrongOperand(op1_type, op2_type, operator)),
        },
    }
}
