use crate::abstract_syntax_tree::{self, Expr, Type, Variable, Operator, UnaryOperator, Literal};

use nom::{
    Err, IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::satisfy,
    combinator::{all_consuming, map, opt, peek, value},
    error::ParseError,
    multi::many1,
    sequence::tuple,
};

mod parse_expression;
mod parse_program;

pub type Program = abstract_syntax_tree::Program<Expr>;
pub type Function = abstract_syntax_tree::Function<Expr>;
pub type Statement = abstract_syntax_tree::Statement<Expr>;
pub type Expression = abstract_syntax_tree::Expression<Expr>;

/// Parse a complete program
pub fn parse<'a, E: ParseError<&'a str> + 'a>(input: &'a str) -> Result<Program, Err<E>> {
    Ok(all_consuming(parse_program::program)(input)?.1)
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

/// Parses 1 or more whitespace characters
fn whitespace<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, (), E> {
    let chars = " \t\r\n";
    map(
        many1(alt((
            take_while1(move |c| chars.contains(c)),
            parse_comment,
        ))),
        |_| (),
    )(i)
}

/// Parse a single comment
fn parse_comment<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let (i, _) = tag("//")(i)?;
    let (i, comment) = take_while1(move |c| c != '\n')(i)?;
    let (i, _) = tag("\n")(i)?;
    Ok((i, comment))
}

/// Optionally parses whitespace
fn optional_ws<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, (), E> {
    map(opt(whitespace), |_| ())(i)
}

/// Parses a variable or function indentifier
/// Can't start with upper case because that is reserved for types
fn identifier<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    // First letter should be lowercase, exit with error if not
    peek(satisfy(|c| c.is_lowercase())).parse(i)?;
    // Parse alphanumeric or underscore
    take_while1(move |c: char| c == '_' || c.is_alphanumeric())(i)
}

/// Parses one of the supported types
fn type_<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Type, E> {
    alt((
        map(tuple((tag("&"), type_)), |(_, type_)| Type::Pointer(Box::new(type_))),
        value(Type::Bool, tag("Bool")),
        value(Type::Int, tag("Int")),
        value(Type::Void, tag("Void")),
        value(Type::Char, tag("Char")),
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
                        Statement::Expression(Expr(Expression::FunctionCall(
                            "print".to_string(),
                            vec![Expr(Expression::Literal(Literal::String(
                                "Hello world!".to_string()
                            )))]
                        ))),
                        Statement::Return(Expr(Expression::Literal(Literal::Int(0))))
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

            // This is a comment
            // Another comment
            Int function(Int arg1, Bool arg2) {
                Int var2;
                if (arg2) {
                    var2 = 2;
                }
                // Ignore this!
                // And this!
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
                            condition: Expr(Expression::Var("arg2".to_string())),
                            then_branch: vec![Statement::Expression(Expr(Expression::BinaryOp(
                                Operator::Assignment,
                                Box::new(Expr(Expression::Var("var2".to_string()))),
                                Box::new(Expr(Expression::Literal(Literal::Int(2)))),
                            )))],
                            else_branch: vec![Statement::Expression(Expr(Expression::BinaryOp(
                                Operator::Assignment,
                                Box::new(Expr(Expression::Var("var2".to_string()))),
                                Box::new(Expr(Expression::Literal(Literal::Int(3)))),
                            )))]
                        },
                        Statement::While {
                            condition: Expr(Expression::Var("arg2".to_string())),
                            body: vec![Statement::Expression(Expr(Expression::BinaryOp(
                                Operator::Assignment,
                                Box::new(Expr(Expression::Var("arg2".to_string()))),
                                Box::new(Expr(Expression::Literal(Literal::Bool(false)))),
                            )))]
                        },
                        Statement::Expression(Expr(Expression::BuiltInFunctionCall(
                            "syscall".to_string(),
                            vec![
                                Expr(Expression::Literal(Literal::Int(1))),
                                Expr(Expression::Literal(Literal::Int(2)))
                            ]
                        ))),
                        Statement::Return(Expr(Expression::Literal(Literal::Int(4))))
                    ]
                }],
                variables: vec![Variable {
                    type_: Type::Int,
                    identifier: "var1".to_string()
                }],
            })
        );
    }

    #[test]
    fn test_pointer_type() {
        let test_string = r#"&&Int"#;
        let res: Result<_, Err<Error<_>>> = type_(&test_string);
        assert_eq!(
            res,
            Ok(("", Type::Pointer(Box::new(Type::Pointer(Box::new(Type::Int))))))
        );
    }
}
