#[derive(Debug)]
pub struct Program {
    functions: Vec<Function>,
    variables: Vec<Variable>,
}

pub type Indentifier = String;

#[derive(Debug)]
pub struct Function {
    return_type: Type,
    arguments: Vec<Variable>,
    indentifier: Indentifier,
    body: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Declaration(Variable),
    Expression(Expression),
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Vec<Statement>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    Return(Expression),
}

#[derive(Debug)]
pub enum Operator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Modulo,
    And,
    Or,
    Xor,
    LessEq,
    Less,
    GreaterEquals,
    Greater,
    Equals,
    NotEqual,
    Assignment,
}

#[derive(Debug)]
pub enum Literal {
    Bool(bool),
    Int(i32),
}

#[derive(Debug)]
pub enum Expression {
    Literal(Literal),
    Var(Indentifier),
    Operator(Operator, Box<Expression>, Box<Expression>),
    FunctionCall(Indentifier, Vec<Expression>),
}

#[derive(Debug)]
pub struct Variable {
    type_: Type,
    identifier: Indentifier,
}

#[derive(Debug)]
pub enum Type {
    Bool,
    Int,
    Void,
}
