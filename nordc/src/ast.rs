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
    Neg,
    And,
    Or,
}

#[derive(Debug)]
pub enum Expr {
    IfElse(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    Constant(Atom),
    Block(Vec<Expr>),
    FunctionCall(Box<Expr>, Box<Expr>),

    // Unary
    UnaryOp(Opcode, Box<Expr>),
    // Binary
    BinaryOp(Box<Expr>, Opcode, Box<Expr>),
}

#[derive(Debug)]
pub enum Atom {
    Num(i64),
    Boolean(bool),
    Identifier(String),
}
