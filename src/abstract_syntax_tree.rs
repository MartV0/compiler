#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub variables: Vec<Variable>,
}

pub type Indentifier = String;

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub return_type: Type,
    pub arguments: Vec<Variable>,
    pub indentifier: Indentifier,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Modulo,
    And,
    Or,
    // TODO: Xor,
    LessEq,
    Less,
    GreaterEquals,
    Greater,
    Equals,
    NotEqual,
    Assignment,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Bool(bool),
    Int(i64),
    String(String)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal(Literal),
    Var(Indentifier),
    // TODO: negation?
    Operator(Operator, Box<Expression>, Box<Expression>),
    FunctionCall(Indentifier, Vec<Expression>),
    // Call to some built in language construct, like syscall
    BuiltInFunctionCall(Indentifier, Vec<Expression>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Variable {
    pub type_: Type,
    pub identifier: Indentifier,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Bool,
    Int,
    Void,
    String
}
