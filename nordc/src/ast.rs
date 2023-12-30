#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Not,
    Neg,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Let(String, Box<Expr>),
    IfElse(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    Constant(Atom),
    Block(Vec<Expr>),
    Call(Box<Expr>, Option<Box<Expr>>),
    Lambda(Option<String>, Box<Expr>),

    Array(Vec<Expr>),
    Object(Vec<(String, Expr)>),

    Index(Box<Expr>, Box<Expr>),
    Member(Box<Expr>, String),

    // Unary
    UnaryOp(Opcode, Box<Expr>),
    // Binary
    BinaryOp(Box<Expr>, Opcode, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Atom {
    Num(i64),
    Boolean(bool),
    Identifier(String),
    String(String),
}
