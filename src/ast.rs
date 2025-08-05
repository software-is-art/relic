use crate::types::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Value(ValueDeclaration),
    Function(FunctionDeclaration),
    Method(MethodDeclaration),
    Relation(RelationDeclaration),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueDeclaration {
    pub name: String,
    pub parameter: Parameter,
    pub body: ValueBody,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    pub name: String,
    pub parameters: Vec<ParameterWithGuard>,
    pub return_type: Type,
    pub body: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDeclaration {
    pub name: String,
    pub parameters: Vec<ParameterWithGuard>,
    pub return_type: Type,
    pub body: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelationDeclaration {
    pub name: String,
    pub fields: Vec<RelationField>,
    pub constraints: Vec<RelationConstraint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelationField {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationConstraint {
    Key(Vec<String>),           // Primary key fields
    Unique(Vec<String>),        // Unique constraint fields
    Foreign {
        field: String,
        references_relation: String,
        references_field: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterWithGuard {
    pub name: String,
    pub ty: Type,
    pub guard: Option<Expression>,
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
    FunctionCall(String, Vec<Expression>),
    MemberAccess(Box<Expression>, String),
    MethodCall(Box<Expression>, String, Vec<Expression>),
    Comparison(ComparisonOp, Box<Expression>, Box<Expression>),
    Pipeline(Box<Expression>, Box<Expression>),
    Let(String, Box<Expression>, Box<Expression>), // let name = value in body
    Match(Box<Expression>, Vec<MatchArm>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Constructor(String, String), // ValueType(binding)
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    And,
    Or,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
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
