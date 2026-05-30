#[derive(Debug, PartialEq, Clone)]
pub struct Program<ExpressionType> {
    pub functions: Vec<Function<ExpressionType>>,
    pub variables: Vec<Variable>,
}

pub type Indentifier = String;

#[derive(Debug, PartialEq, Clone)]
pub struct Function<ExpressionType> {
    pub return_type: Type,
    pub arguments: Vec<Variable>,
    pub indentifier: Indentifier,
    pub body: Vec<Statement<ExpressionType>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<ExpressionType> {
    Declaration(Variable),
    Expression(ExpressionType),
    If {
        condition: ExpressionType,
        then_branch: Vec<Statement<ExpressionType>>,
        else_branch: Vec<Statement<ExpressionType>>,
    },
    While {
        condition: ExpressionType,
        body: Vec<Statement<ExpressionType>>,
    },
    Return(ExpressionType),
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
    ArraySubScript
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOperator {
    Dereference,
    AddressOf,
    Negation
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Bool(bool),
    Int(i64),
    String(String),
}

// type Exp = Expression<Exp>;
#[derive(Debug, PartialEq, Clone)]
pub struct Expr (pub Expression<Expr>);

#[derive(Debug, PartialEq, Clone)]
pub enum Expression<ExpressionType> {
    Literal(Literal),
    Var(Indentifier),
    BinaryOp(Operator, Box<ExpressionType>, Box<ExpressionType>),
    UnaryOp(UnaryOperator, Box<ExpressionType>),
    FunctionCall(Indentifier, Vec<ExpressionType>),
    // Call to some built in language construct, like syscall
    BuiltInFunctionCall(Indentifier, Vec<ExpressionType>),
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
    Char,
    Pointer(Box<Type>)
}

pub fn map_from_expr(exprs: Vec<Expr>) -> Vec<Expression<Expr>> {
    exprs.into_iter().map(| expr | expr.0).collect()
}

pub fn map_to_expr(exprs: Vec<Expression<Expr>>) -> Vec<Expr> {
    exprs.into_iter().map(| expr | Expr(expr)).collect()
}
