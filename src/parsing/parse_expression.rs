use super::*;

use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, escaped_transform, is_not},
    character::complete::digit1,
    combinator::{map, value},
    error::ParseError,
    multi::separated_list0,
    sequence::tuple,
};

/// Parses an expression
/// Entry point for parsing expression
pub fn expression<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Expression, E> {
    expression6(i)
}

/// Parses simple expression, containing single value, or in parenthesis
fn expression_simple<'a, E: ParseError<&'a str> + 'a>(
    i: &'a str,
) -> IResult<&'a str, Expression, E> {
    alt((
        value(Expression::Literal(Literal::Bool(true)), tag("True")),
        value(Expression::Literal(Literal::Bool(false)), tag("False")),
        map(string_literal, |s| Expression::Literal(Literal::String(s))),
        map(digit1, |str| {
            Expression::Literal(Literal::Int(str::parse(str).expect("should be parseble")))
        }),
        function_call,
        builtin_function_call,
        map(identifier, |ident| Expression::Var(ident.to_string())),
        parenthesised(expression),
    ))(i)
}

/// Parses a string literal
fn string_literal<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, String, E> {
    let (i, _) = optional_ws(i)?;
    let (i, _) = tag("\"")(i)?;
    let (i, string_content) = escaped_transform(
        is_not("\\\""), 
        '\\',
        alt((
          value("\\", tag("\\")),
          value("\"", tag("\"")),
          value("\n", tag("n")),
          value("\t", tag("t")),
          value("\r", tag("r"))
        ))
    )(i)?;
    let (i, _) = tag("\"")(i)?;
    Ok((i, string_content.to_string()))
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

/// Parses a builtin function call expression
fn builtin_function_call<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Expression, E> {
    let (i, _) = optional_ws(i)?;
    let (i, ident) = identifier(i)?;
    let (i, _) = tag("!")(i)?;
    let (i, _) = optional_ws(i)?;
    let (i, arguments) = parenthesised(separated_list0(
        tuple((optional_ws, tag(","), optional_ws)),
        expression,
    ))(i)?;
    Ok((i, Expression::BuiltInFunctionCall(ident.to_string(), arguments)))
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
        Ok((i, fold_ops_exprs(ops, exprs, Associativity::Left)))
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
        Ok((i, fold_ops_exprs(ops, exprs, Associativity::Right)))
    }
}

enum Associativity {
    Left,
    Right
}

/// Fold the exprs into a single one using the operators
fn fold_ops_exprs(ops: impl Iterator<Item=Operator>, mut exprs: impl Iterator<Item=Expression>, associativity: Associativity) -> Expression {
    let expr1 = exprs.next().expect("should be at least one expression");
    std::iter::zip(ops, exprs).fold(expr1, |expr1, (op, expr2)| {
        match associativity {
            Associativity::Left => Expression::Operator(op, Box::new(expr1), Box::new(expr2)),
            Associativity::Right => Expression::Operator(op, Box::new(expr2), Box::new(expr1))
        }
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

/// Parse operators with lowest associativity
fn bin_operator1<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Operator, E> {
    alt((
        value(Operator::Division, tag("/")),
        value(Operator::Multiplication, tag("*")),
        value(Operator::Modulo, tag("%")),
    ))(i)
}

/// Parse operators with associativity 2
fn bin_operator2<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Operator, E> {
    alt((
        value(Operator::Addition, tag("+")),
        value(Operator::Subtraction, tag("-")),
    ))(i)
}

/// Parse operators with associativity 3
fn bin_operator3<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Operator, E> {
    alt((
        value(Operator::LessEq, tag("<=")),
        value(Operator::Less, tag("<")),
        value(Operator::Greater, tag(">")),
        value(Operator::GreaterEquals, tag(">=")),
    ))(i)
}

/// Parse operators with associativity 4
fn bin_operator4<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Operator, E> {
    alt((
        value(Operator::Equals, tag("==")),
        value(Operator::NotEqual, tag("!=")),
    ))(i)
}

/// Parse operators with associativity 5
fn bin_operator5<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Operator, E> {
    alt((
        value(Operator::And, tag("&&")),
        value(Operator::Or, tag("||")),
    ))(i)
}

/// Parse operators with associativity 6
fn bin_operator6<'a, E: ParseError<&'a str> + 'a>(i: &'a str) -> IResult<&'a str, Operator, E> {
    value(Operator::Assignment, tag("="))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::Error;

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

    #[test]
    fn test_str_expression() {
        let test_string = r#""HELLO there \r \n \\ \"""#;
        let res: IResult<_, _, Error<_>> = expression(&test_string);
        assert_eq!(
            res,
            Ok((
                "",
                Expression::Literal(Literal::String("HELLO there \r \n \\ \"".to_string()))
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
}
