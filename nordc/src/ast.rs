#[derive(Debug)]
pub enum Expr {
    Application(String, Vec<Expr>),
    IfElse(Box<Expr>, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>),
    Constant(Atom),
    Quote(Box<Expr>),
}

#[derive(Debug)]
pub enum Atom {
    Num(i64),
    Boolean(bool),
    Builtin(String),
    Keyword(String),
}
