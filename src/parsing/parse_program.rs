#![allow(clippy::redundant_field_names)]

use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    error::ParseError,
    multi::{many0, separated_list0},
    sequence::tuple,
};

use super::*;
use super::parse_expression::expression;

/// Parse the complete program file
/// Main entry point for the parsing
pub fn program<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Program, E> {
    let (i, contents) = many0(alt((
        map(declaration, ProgramContent::Var),
        map(function, ProgramContent::Func),
    )))(i)?;

    let mut functions = vec![];
    let mut variables = vec![];

    for content in contents {
        match content {
            ProgramContent::Func(func) => functions.push(func),
            ProgramContent::Var(var) => variables.push(var),
        }
    }

    let (i, _) = optional_ws(i)?;

    Ok((
        i,
        Program {
            functions: functions,
            variables: variables,
        },
    ))
}

/// Parses a variable declaration followed by semicolon
fn declaration<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Variable, E> {
    let (i, var) = variable(i)?;
    let (i, _) = optional_ws(i)?;
    let (i, _) = tag(";")(i)?;
    Ok((i, var))
}

/// Type temporarily used because functions and variables declaration can be mixed
enum ProgramContent {
    Func(Function),
    Var(Variable),
}

/// Parses the function
fn function<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Function, E> {
    let (i, _) = optional_ws(i)?;
    let (i, return_type) = type_(i)?;
    let (i, _) = whitespace(i)?;
    let (i, ident) = identifier(i)?;
    let (i, _) = optional_ws(i)?;
    let (i, arguments) =
        parenthesised(separated_list0(tuple((optional_ws, tag(","))), variable))(i)?;
    let (i, body) = block(i)?;
    Ok((
        i,
        Function {
            return_type,
            arguments: arguments,
            indentifier: ident.to_string(),
            body: body,
        },
    ))
}

/// Parses a block: multiple statements contained within curly braces
fn block<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Vec<Statement>, E> {
    let (i, _) = optional_ws(i)?;
    let (i, _) = tag("{")(i)?;
    let (i, statements) = many0(statement)(i)?;
    let (i, _) = optional_ws(i)?;
    let (i, _) = tag("}")(i)?;
    Ok((i, statements))
}

/// Parses a statement
fn statement<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Statement, E> {
    alt((
        map(declaration, Statement::Declaration),
        map(expression_stmt, Statement::Expression),
        if_stmt,
        while_stmt,
        return_stmt,
    ))(i)
}

/// Parses an if statement
/// TODO: make else branch optional
fn if_stmt<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Statement, E> {
    let (i, _) = optional_ws(i)?;
    let (i, _) = tag("if")(i)?;
    let (i, _) = optional_ws(i)?;
    let (i, condition) = parenthesised(expression)(i)?;
    let (i, then) = block(i)?;
    let (i, _) = optional_ws(i)?;
    let (i, _) = tag("else")(i)?;
    let (i, else_) = block(i)?;
    Ok((
        i,
        Statement::If {
            condition: condition,
            then_branch: then,
            else_branch: else_,
        },
    ))
}

/// Parses an while statement
fn while_stmt<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Statement, E> {
    let (i, _) = optional_ws(i)?;
    let (i, _) = tag("while")(i)?;
    let (i, _) = optional_ws(i)?;
    let (i, condition) = parenthesised(expression)(i)?;
    let (i, body) = block(i)?;
    Ok((
        i,
        Statement::While {
            condition: condition,
            body: body,
        },
    ))
}

/// Parses an return statement
fn return_stmt<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Statement, E> {
    let (i, _) = optional_ws(i)?;
    let (i, _) = tag("return")(i)?;
    let (i, _) = whitespace(i)?;
    let (i, expr) = expression(i)?;
    let (i, _) = optional_ws(i)?;
    let (i, _) = tag(";")(i)?;
    Ok((i, Statement::Return(expr)))
}

/// Parses an expression statement
fn expression_stmt<'a, E: ParseError<&'a str> + 'a + 'a>(
    i: &'a str,
) -> IResult<&'a str, Expression, E> {
    let (i, _) = optional_ws(i)?;
    let (i, expr) = expression(i)?;
    let (i, _) = optional_ws(i)?;
    let (i, _) = tag(";")(i)?;
    Ok((i, expr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::Error;

    #[test]
    fn test_variable() {
        let test_string = " Bool yes ; ";
        let res: IResult<_, _, Error<_>> = declaration(&test_string);
        assert_eq!(
            res,
            Ok((
                " ",
                (Variable {
                    type_: Type::Bool,
                    identifier: "yes".to_string()
                })
            ))
        );
    }

    #[test]
    fn test_variable_program() {
        let test_string = " Bool yes ; Int    no; 
            Int yes2;\t";
        let res: IResult<_, _, Error<_>> = program(&test_string);
        assert_eq!(
            res,
            Ok((
                "",
                (Program {
                    variables: vec![
                        Variable {
                            type_: Type::Bool,
                            identifier: "yes".to_string()
                        },
                        Variable {
                            type_: Type::Int,
                            identifier: "no".to_string()
                        },
                        Variable {
                            type_: Type::Int,
                            identifier: "yes2".to_string()
                        },
                    ],
                    functions: vec![]
                })
            ))
        );
    }

    #[test]
    fn test_function() {
        let test_string = "
            Int function(Int arg1, Bool arg2) {
                Int var2;
                if (arg2) {
                    var2 = 2;
                }
                else {
                    var2 = 3;
                }
                while(arg2) {
                    arg2 = False;
                }
                return 4;
            }";
        let res: Result<_, Err<Error<_>>> = function(&test_string);
        assert_eq!(
            res,
            Ok((
                "",
                Function {
                    return_type: Type::Int,
                    arguments: vec![
                        Variable {
                            type_: Type::Int,
                            identifier: "arg1".to_string()
                        },
                        Variable {
                            type_: Type::Bool,
                            identifier: "arg2".to_string()
                        }
                    ],
                    indentifier: "function".to_string(),
                    body: vec![
                        Statement::Declaration(Variable {
                            type_: Type::Int,
                            identifier: "var2".to_string()
                        }),
                        Statement::If {
                            condition: Expression::Var("arg2".to_string()),
                            then_branch: vec![Statement::Expression(Expression::Operator(
                                Operator::Assignment,
                                Box::new(Expression::Var("var2".to_string())),
                                Box::new(Expression::Literal(Literal::Int(2))),
                            ))],
                            else_branch: vec![Statement::Expression(Expression::Operator(
                                Operator::Assignment,
                                Box::new(Expression::Var("var2".to_string())),
                                Box::new(Expression::Literal(Literal::Int(3))),
                            ))]
                        },
                        Statement::While {
                            condition: Expression::Var("arg2".to_string()),
                            body: vec![Statement::Expression(Expression::Operator(
                                Operator::Assignment,
                                Box::new(Expression::Var("arg2".to_string())),
                                Box::new(Expression::Literal(Literal::Bool(false))),
                            ))]
                        },
                        Statement::Return(Expression::Literal(Literal::Int(4)))
                    ]
                }
            ))
        );
    }

    #[test]
    fn test_if() {
        let test_string = "
            if (arg2) {
                var2 = 2;
            }
            else {
                var2 = 3;
            }";
        let res: Result<_, Err<Error<_>>> = if_stmt(&test_string);
        assert_eq!(
            res,
            Ok((
                "",
                Statement::If {
                    condition: Expression::Var("arg2".to_string()),
                    then_branch: vec![Statement::Expression(Expression::Operator(
                        Operator::Assignment,
                        Box::new(Expression::Var("var2".to_string())),
                        Box::new(Expression::Literal(Literal::Int(2))),
                    ))],
                    else_branch: vec![Statement::Expression(Expression::Operator(
                        Operator::Assignment,
                        Box::new(Expression::Var("var2".to_string())),
                        Box::new(Expression::Literal(Literal::Int(3))),
                    ))]
                }
            ))
        );
    }

    #[test]
    fn test_block() {
        let test_string = "{ var2 = 2; }";
        let res: Result<_, Err<Error<_>>> = block(&test_string);
        assert_eq!(
            res,
            Ok((
                "",
                vec![Statement::Expression(Expression::Operator(
                    Operator::Assignment,
                    Box::new(Expression::Var("var2".to_string())),
                    Box::new(Expression::Literal(Literal::Int(2))),
                ))]
            ))
        );
    }

    #[test]
    fn test_statement_expr() {
        let test_string = "var2 = 2; ";
        let res: Result<_, Err<Error<_>>> = statement(&test_string);
        assert_eq!(
            res,
            Ok((
                " ",
                Statement::Expression(Expression::Operator(
                    Operator::Assignment,
                    Box::new(Expression::Var("var2".to_string())),
                    Box::new(Expression::Literal(Literal::Int(2))),
                ))
            ))
        );
    }
}
