use crate::abstract_syntax_tree::*;

use nom::{
    Err, IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::satisfy,
    combinator::{map, peek, value},
    error::{ContextError, Error, ErrorKind, ParseError, context},
    multi::{many0, separated_list0},
    sequence::tuple,
};

pub fn parse(input: &str) -> Program {
    todo!("parse")
}

/// Parses 1 or more whitespace characters
fn whitespace<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";

    take_while1(move |c| chars.contains(c))(i)
}

/// Optionally parses whitespace
fn optional_ws<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";

    take_while(move |c| chars.contains(c))(i)
}

/// Parses a variable or function indentifier
/// Can't start with upper case because that is reserved for types
fn identifier<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    // First letter should be lowercase
    let (i, _) = peek(satisfy(|c| c.is_lowercase())).parse(i)?;
    // Parse alphanumeric or underscore
    take_while1(move |c: char| (c == '_' || c.is_alphanumeric()))(i)
}

/// Parses one of the supported types
fn type_<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Type, E> {
    alt((
        value(Type::Bool, tag("Bool")),
        value(Type::Int, tag("Int")),
        value(Type::Void, tag("Void")),
    ))(i)
}

/// Parses a type and an indentifier, separated by whitespace, and optional whitespace in front
fn declaration<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Variable, E> {
    let (i, _) = optional_ws(i)?;
    let (i, type_signature) = type_(i)?;
    let (i, _) = whitespace(i)?;
    let (i, ident) = identifier(i)?;
    Ok((
        i,
        Variable {
            type_: type_signature,
            identifier: ident.to_string(),
        },
    ))
}

/// Parses a variable declaration followed by semicolon
fn variable<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Variable, E> {
    let (i, var) = declaration(i)?;
    let (i, _) = optional_ws(i)?;
    let (i, _) = tag(";")(i)?;
    Ok((i, var))
}

enum ProgramContent {
    Func(Function),
    Var(Variable),
}

// Parse the complete program file
fn program<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Program, E> {
    let (i, contents) = many0(alt((
        map(variable, |var| ProgramContent::Var(var)),
        map(function, |func| ProgramContent::Func(func)),
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

/// Parses the function
fn function<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Function, E> {
    let (i, _) = optional_ws(i)?;
    let (i, return_type) = type_(i)?;
    let (i, _) = whitespace(i)?;
    let (i, ident) = identifier(i)?;
    let (i, _) = optional_ws(i)?;
    let (i, _) = tag("(")(i)?;
    let (i, arguments) = separated_list0(tuple((optional_ws, tag(","))), declaration)(i)?;
    let (i, _) = tag(")")(i)?;
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
fn block<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Vec<Statement>, E> {
    let (i, _) = optional_ws(i)?;
    let (i, _) = tag("{")(i)?;
    let (i, statements) = many0(statement)(i)?;
    let (i, _) = optional_ws(i)?;
    let (i, _) = tag("}")(i)?;
    Ok((
        i,
        statements
    ))
}

/// Parses a statement
fn statement<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Statement, E> {
    alt((
        map(variable, |var| Statement::Declaration(var)),
        map(expression_stmt, |expr| Statement::Expression(expr)),
        if_stmt,
        while_stmt,
        return_stmt,
    ))(i)
}

/// Parses an if statement
fn if_stmt<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Statement, E> {
    todo!()
}

/// Parses an while statement
fn while_stmt<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Statement, E> {
    todo!()
}

/// Parses an return statement
fn return_stmt<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Statement, E> {
    todo!()
}

/// Parses an expression statement
fn expression_stmt<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Expression, E> {
    let (i, expr) = expression(i)?;
    let (i, _) = optional_ws(i)?;
    let (i, _) = tag(";")(i)?;
    Ok((i, expr))
}

/// Parses an expression
fn expression<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Expression, E> {
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable() {
        let test_string = " Bool yes ; ";
        let res: IResult<_, _, Error<_>> = variable(&test_string);
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
                "\t",
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
}
