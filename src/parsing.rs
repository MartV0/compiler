use crate::abstract_syntax_tree::*;

use nom::{
    Err, IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::satisfy,
    combinator::{all_consuming, peek, value},
    error::ParseError,
};

mod parse_expression;
mod parse_program;

/// Parse a complete program
pub fn parse<'a, E: ParseError<&'a str> + 'a>(input: &'a str) -> Result<Program, Err<E>> {
    Ok(all_consuming(parse_program::program)(input)?.1)
}

/// Parses 1 or more whitespace characters
fn whitespace<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";
    take_while1(move |c| chars.contains(c))(i)
}

/// Parser with parenthesis around it, with optional whitespace
fn parenthesised<'a, E: ParseError<&'a str> + 'a, F, R>(
    mut p: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, R, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, R, E> + 'a,
{
    move |i| {
        let (i, _) = tag("(")(i)?;
        let (i, _) = optional_ws(i)?;
        let (i, res) = p(i)?;
        let (i, _) = optional_ws(i)?;
        let (i, _) = tag(")")(i)?;
        Ok((i, res))
    }
}

/// Optionally parses whitespace
fn optional_ws<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";
    take_while(move |c| chars.contains(c))(i)
}

/// Parses a variable or function indentifier
/// Can't start with upper case because that is reserved for types
fn identifier<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    // First letter should be lowercase, exit with error if not
    peek(satisfy(|c| c.is_lowercase())).parse(i)?;
    // Parse alphanumeric or underscore
    take_while1(move |c: char| (c == '_' || c.is_alphanumeric()))(i)
}

/// Parses one of the supported types
fn type_<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Type, E> {
    alt((
        value(Type::Bool, tag("Bool")),
        value(Type::Int, tag("Int")),
        value(Type::Void, tag("Void")),
        value(Type::String, tag("String")),
    ))(i)
}

/// Parses a type and an indentifier, separated by whitespace, and optional whitespace in front
fn variable<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Variable, E> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::Error;

    #[test]
    fn test_hello_world() {
        let test_string = r#"
            Int main() {
                print("Hello world!");
                return 0;
            }
        "#;
        let res: Result<_, Err<Error<_>>> = parse(&test_string);
        assert_eq!(
            res,
            Ok(Program {
                functions: vec![Function {
                    return_type: Type::Int,
                    arguments: vec![],
                    indentifier: "main".to_string(),
                    body: vec![
                        Statement::Expression(Expression::FunctionCall(
                            "print".to_string(),
                            vec![Expression::Literal(Literal::String(
                                "Hello world!".to_string()
                            ))]
                        )),
                        Statement::Return(Expression::Literal(Literal::Int(0)))
                    ]
                }],
                variables: vec![],
            })
        );
    }

    #[test]
    fn test_full_parser() {
        let test_string = "
            Int var1;

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

                syscall!(1, 2);

                return 4;
            }
        ";
        let res: Result<_, Err<Error<_>>> = parse(&test_string);
        assert_eq!(
            res,
            Ok(Program {
                functions: vec![Function {
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
                        Statement::Expression(Expression::BuiltInFunctionCall(
                            "syscall".to_string(),
                            vec![
                                Expression::Literal(Literal::Int(1)),
                                Expression::Literal(Literal::Int(2))
                            ]
                        )),
                        Statement::Return(Expression::Literal(Literal::Int(4)))
                    ]
                }],
                variables: vec![Variable {
                    type_: Type::Int,
                    identifier: "var1".to_string()
                }],
            })
        );
    }
}
