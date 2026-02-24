use crate::abstract_syntax_tree::*;


use nom::{
  branch::alt,
  bytes::complete::{escaped, tag, take_while},
  character::complete::{alphanumeric1 as alphanumeric, char, one_of},
  combinator::{cut, map, opt, value},
  error::{context, ContextError, ErrorKind, ParseError},
  multi::separated_list0,
  number::complete::double,
  sequence::{delimited, preceded, separated_pair, terminated},
  Err, IResult, Parser,
};

pub fn parse(input: &str) -> Program {
    todo!("parse")
}

fn whitespace<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
  let chars = " \t\r\n";

  take_while(move |c| chars.contains(c))(i)
}
