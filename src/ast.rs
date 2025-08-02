use crate::types::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Value(ValueDeclaration),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueDeclaration {
    pub name: String,
    pub parameter: Parameter,
    pub body: ValueBody,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueBody {
    pub validate: Option<Expression>,
    pub normalize: Option<Expression>,
    pub unique: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Binary(BinaryOp, Box<Expression>, Box<Expression>),
    Unary(UnaryOp, Box<Expression>),
    Literal(Literal),
    Identifier(String),
    MemberAccess(Box<Expression>, String),
    MethodCall(Box<Expression>, String, Vec<Expression>),
    Comparison(ComparisonOp, Box<Expression>, Box<Expression>),
    Pipeline(Box<Expression>, Box<Expression>),
    Let(String, Box<Expression>, Box<Expression>), // let name = value in body
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    And,
    Or,
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
    Minus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOp {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Contains,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Integer(i64),
    Boolean(bool),
}
