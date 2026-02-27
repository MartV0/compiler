use crate::abstract_syntax_tree::*;

use nom::{
    Err, IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{digit1, satisfy},
    combinator::{all_consuming, map, peek, value},
    error::{Error, ParseError},
    multi::{many0, separated_list0},
    sequence::tuple,
};

/// Parse a complete program
pub fn parse<'a, E: ParseError<&'a str> + 'a>(input: &'a str) -> Result<Program, Err<E>> {
    Ok(all_consuming(program)(input)?.1)
}

/// Parses 1 or more whitespace characters
fn whitespace<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";
    take_while1(move |c| chars.contains(c))(i)
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

/// Parse the complete program file
fn program<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Program, E> {
    let (i, contents) = many0(alt((
        map(declaration, |var| ProgramContent::Var(var)),
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
        map(declaration, |var| Statement::Declaration(var)),
        map(expression_stmt, |expr| Statement::Expression(expr)),
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

/// Parses simple expression, containing single value, or in parenthesis
fn expression_simple<'a, E: ParseError<&'a str> + 'a>(
    i: &'a str,
) -> IResult<&'a str, Expression, E> {
    alt((
        value(Expression::Literal(Literal::Bool(true)), tag("True")),
        value(Expression::Literal(Literal::Bool(false)), tag("False")),
        map(digit1, |str| {
            Expression::Literal(Literal::Int(str::parse(str).expect("should be parseble")))
        }),
        function_call,
        map(identifier, |ident| Expression::Var(ident.to_string())),
        parenthesised(expression),
    ))(i)
}

/// Parses function call expression
fn function_call<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Expression, E> {
    let (i, _) = optional_ws(i)?;
    let (i, ident) = identifier(i)?;
    let (i, _) = optional_ws(i)?;
    let (i, arguments) = parenthesised(separated_list0(
        tuple((optional_ws, tag(","), optional_ws)),
        expression,
    ))(i)?;
    Ok((i, Expression::FunctionCall(ident.to_string(), arguments)))
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

/// Parse left associatively, based on a given operator and expression parser
fn left_associative<'a, E: ParseError<&'a str> + 'a, Fe, Fo>(
    operator: Fo,
    expression: Fe,
) -> impl FnMut(&'a str) -> IResult<&'a str, Expression, E>
where
    Fo: FnMut(&'a str) -> IResult<&'a str, Operator, E> + 'a + Copy,
    Fe: FnMut(&'a str) -> IResult<&'a str, Expression, E> + 'a + Copy,
{
    move |i| {
        let (i, (ops, exprs)) = accumulate_ops_exprs(i, operator, expression)?;
        let ops = ops.into_iter();
        let exprs = exprs.into_iter();
        Ok((i, fold_ops_exprs(ops, exprs)))
    }
}

/// Parse right associatively, based on a given operator and expression parser
fn right_associative<'a, E: ParseError<&'a str> + 'a, Fe, Fo>(
    operator: Fo,
    expression: Fe,
) -> impl FnMut(&'a str) -> IResult<&'a str, Expression, E>
where
    Fo: FnMut(&'a str) -> IResult<&'a str, Operator, E> + 'a + Copy,
    Fe: FnMut(&'a str) -> IResult<&'a str, Expression, E> + 'a + Copy,
{
    move |i| {
        let (i, (ops, exprs)) = accumulate_ops_exprs(i, operator, expression)?;
        let ops = ops.into_iter().rev();
        let exprs = exprs.into_iter().rev();
        Ok((i, fold_ops_exprs(ops, exprs)))
    }
}

/// Fold the exprs into a single one using the operators
fn fold_ops_exprs(ops: impl Iterator<Item=Operator>, mut exprs: impl Iterator<Item=Expression>) -> Expression {
    let expr1 = exprs.next().expect("should be at least one expression");
    std::iter::zip(ops, exprs).fold(expr1, |expr1, (op, expr2)| {
        Expression::Operator(op, Box::new(expr1), Box::new(expr2))
    })
}

/// Parse a list of experessions seperated by operators, accumulating them in seperate lists
fn accumulate_ops_exprs<'a, E: ParseError<&'a str> + 'a, Fo, Fe>(
    i: &'a str,
    operator: Fo,
    mut expression: Fe,
) -> IResult<&'a str, (Vec<Operator>, Vec<Expression>), E>
where
    Fo: FnMut(&'a str) -> IResult<&'a str, Operator, E> + 'a + Copy,
    Fe: FnMut(&'a str) -> IResult<&'a str, Expression, E> + 'a + Copy,
{
    let (mut i, e1) = expression(i)?;
    let mut exprs = vec![e1];
    let mut ops = vec![];
    while let Ok((i2, (_, op, _, expr))) =
        tuple((optional_ws, operator, optional_ws, expression))(i)
    {
        i = i2;
        exprs.push(expr);
        ops.push(op);
    }
    Ok((i, (ops, exprs)))
}

/// Parses an expression
fn expression<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Expression, E> {
    expression6(i)
}

/// Parse operators with highest associativity first
/// Right associative because this level only contains assign operator
fn expression6<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Expression, E> {
    right_associative(bin_operator6, expression5)(i)
}

/// Parse operators with associativity 5
/// Left associative: && ||
fn expression5<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Expression, E> {
    left_associative(bin_operator5, expression4)(i)
}

/// Parse operators with associativity 4
/// Left associative: == !=
fn expression4<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Expression, E> {
    left_associative(bin_operator4, expression3)(i)
}

/// Parse operators with associativity 3
/// Left associative: <= < > >=
fn expression3<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Expression, E> {
    left_associative(bin_operator3, expression2)(i)
}

/// Parse operators with associativity 2
/// Left associative: + -
fn expression2<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Expression, E> {
    left_associative(bin_operator2, expression1)(i)
}

/// Parse operators with associativity 1
/// Left associative: / * %
fn expression1<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Expression, E> {
    left_associative(bin_operator1, expression_simple)(i)
}

// Parse operators with lowest associativity
fn bin_operator1<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Operator, E> {
    alt((
        value(Operator::Division, tag("/")),
        value(Operator::Multiplication, tag("*")),
        value(Operator::Modulo, tag("%")),
    ))(i)
}

// Parse operators with associativity 2
fn bin_operator2<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Operator, E> {
    alt((
        value(Operator::Addition, tag("+")),
        value(Operator::Subtraction, tag("-")),
    ))(i)
}

// Parse operators with associativity 3
fn bin_operator3<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Operator, E> {
    alt((
        value(Operator::LessEq, tag("<=")),
        value(Operator::Less, tag("<")),
        value(Operator::Greater, tag(">")),
        value(Operator::GreaterEquals, tag(">=")),
    ))(i)
}

// Parse operators with associativity 4
fn bin_operator4<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Operator, E> {
    alt((
        value(Operator::Equals, tag("==")),
        value(Operator::NotEqual, tag("!=")),
    ))(i)
}

// Parse operators with associativity 5
fn bin_operator5<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Operator, E> {
    alt((
        value(Operator::And, tag("&&")),
        value(Operator::Or, tag("||")),
    ))(i)
}

// Parse operators with associativity 6
fn bin_operator6<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Operator, E> {
    value(Operator::Assignment, tag("="))(i)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_expression() {
        let test_string = "1+2";
        let res: IResult<_, _, Error<_>> = expression(&test_string);
        assert_eq!(
            res,
            Ok((
                "",
                Expression::Operator(
                    Operator::Addition,
                    Box::new(Expression::Literal(Literal::Int(1))),
                    Box::new(Expression::Literal(Literal::Int(2)))
                )
            ))
        );
    }

    #[test]
    fn test_expression_associativity() {
        let test_string = "1+2+3";
        let res: IResult<_, _, Error<_>> = expression(&test_string);
        assert_eq!(
            res,
            Ok((
                "",
                Expression::Operator(
                    Operator::Addition,
                    Box::new(Expression::Operator(
                        Operator::Addition,
                        Box::new(Expression::Literal(Literal::Int(1))),
                        Box::new(Expression::Literal(Literal::Int(2)))
                    )),
                    Box::new(Expression::Literal(Literal::Int(3)))
                )
            ))
        );
    }

    #[test]
    fn test_expression_associativity2() {
        let test_string = "var=1*2*3+4+5";
        let res: IResult<_, _, Error<_>> = expression(&test_string);
        assert_eq!(
            res,
            Ok((
                "",
                Expression::Operator(
                    Operator::Assignment,
                    Box::new(Expression::Var("var".to_string())),
                    Box::new(Expression::Operator(
                        Operator::Addition,
                        Box::new(Expression::Operator(
                            Operator::Addition,
                            Box::new(Expression::Operator(
                                Operator::Multiplication,
                                Box::new(Expression::Operator(
                                    Operator::Multiplication,
                                    Box::new(Expression::Literal(Literal::Int(1))),
                                    Box::new(Expression::Literal(Literal::Int(2)))
                                )),
                                Box::new(Expression::Literal(Literal::Int(3)))
                            )),
                            Box::new(Expression::Literal(Literal::Int(4)))
                        )),
                        Box::new(Expression::Literal(Literal::Int(5)))
                    )),
                )
            ))
        );
    }

    #[test]
    fn test_expression_function_call() {
        let test_string = "function(arg1, 2)";
        let res: IResult<_, _, Error<_>> = expression(&test_string);
        assert_eq!(
            res,
            Ok((
                "",
                Expression::FunctionCall(
                    "function".to_string(),
                    vec![
                        Expression::Var("arg1".to_string()),
                        Expression::Literal(Literal::Int(2))
                    ]
                )
            ))
        );
    }

    #[test]
    fn test_function_call() {
        let test_string = "function(arg1, 2)";
        let res: IResult<_, _, Error<_>> = function_call(&test_string);
        assert_eq!(
            res,
            Ok((
                "",
                Expression::FunctionCall(
                    "function".to_string(),
                    vec![
                        Expression::Var("arg1".to_string()),
                        Expression::Literal(Literal::Int(2))
                    ]
                )
            ))
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

    #[test]
    fn test_parenthesised_test() {
        let test_string = "(a + b) * (c + d)";
        let res: Result<_, Err<Error<_>>> = expression(&test_string);
        assert_eq!(
            res,
            Ok((
                "",
                Expression::Operator(
                    Operator::Multiplication,
                    Box::new(Expression::Operator(
                        Operator::Addition,
                        Box::new(Expression::Var("a".to_string())),
                        Box::new(Expression::Var("b".to_string())),
                    )),
                    Box::new(Expression::Operator(
                        Operator::Addition,
                        Box::new(Expression::Var("c".to_string())),
                        Box::new(Expression::Var("d".to_string())),
                    )),
                )
            ))
        );
    }
}
