use winnow::{
    ascii::{alpha1, digit1, multispace0},
    combinator::{alt, cut_err, delimited, opt, preceded, repeat},
    error::{ContextError, StrContext},
    prelude::*,
    token::one_of,
};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Expr {
    Constant(Atom),
    Application(Box<Expr>, Vec<Expr>),
    If(Box<Expr>, Box<Expr>),
    IfElse(Box<Expr>, Box<Expr>, Box<Expr>),
    Quote(Vec<Expr>),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Atom {
    Num(i32),
    Keyword(String),
    Boolean(bool),
    BuiltIn(BuiltIn),
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum BuiltIn {
    Plus,
    Minus,
    Times,
    Divide,
    Equal,
    Not,
}

pub fn parse_expr(i: &mut &'_ str) -> PResult<Expr> {
    preceded(
        multispace0,
        alt((parse_constant, parse_application, parse_if, parse_quote)),
    )
    .parse_next(i)
}

pub fn parse_constant(i: &mut &'_ str) -> PResult<Expr> {
    parse_atom.map(Expr::Constant).parse_next(i)
}

fn parse_atom(i: &mut &'_ str) -> PResult<Atom> {
    alt((
        parse_num,
        parse_bool,
        parse_builtin.map(Atom::BuiltIn),
        parse_keyword,
    ))
    .parse_next(i)
}

fn parse_num(i: &mut &'_ str) -> PResult<Atom> {
    digit1
        .try_map(|digit_str: &str| digit_str.parse::<i32>().map(Atom::Num))
        .parse_next(i)
}

fn parse_bool(i: &mut &'_ str) -> PResult<Atom> {
    alt((
        "true".map(|_| Atom::Boolean(true)),
        "false".map(|_| Atom::Boolean(false)),
    ))
    .parse_next(i)
}

fn parse_builtin(i: &mut &'_ str) -> PResult<BuiltIn> {
    alt((parse_builtin_op, "not".map(|_| BuiltIn::Not))).parse_next(i)
}

fn parse_builtin_op(i: &mut &'_ str) -> PResult<BuiltIn> {
    let t = one_of(['+', '-', '*', '/', '=']).parse_next(i)?;
    Ok(match t {
        '+' => BuiltIn::Plus,
        '-' => BuiltIn::Minus,
        '*' => BuiltIn::Times,
        '/' => BuiltIn::Divide,
        '=' => BuiltIn::Equal,
        _ => unreachable!(),
    })
}

fn parse_keyword(i: &mut &'_ str) -> PResult<Atom> {
    preceded(":", cut_err(alpha1))
        .context(StrContext::Label("keyword"))
        .map(|sym_str: &str| Atom::Keyword(sym_str.to_string()))
        .parse_next(i)
}

fn parse_application(i: &mut &'_ str) -> PResult<Expr> {
    let application_inner = (parse_expr, repeat(0.., parse_expr))
        .map(|(head, tail)| Expr::Application(Box::new(head), tail));
    s_exp(application_inner).parse_next(i)
}

fn parse_if(i: &mut &'_ str) -> PResult<Expr> {
    let if_inner = preceded("if", cut_err((parse_expr, parse_expr, opt(parse_expr))))
        .map(|(pred, true_branch, maybe_false_branch)| {
            if let Some(false_branch) = maybe_false_branch {
                Expr::IfElse(
                    Box::new(pred),
                    Box::new(true_branch),
                    Box::new(false_branch),
                )
            } else {
                Expr::If(Box::new(pred), Box::new(true_branch))
            }
        })
        .context(StrContext::Label("if expression"));
    s_exp(if_inner).parse_next(i)
}

fn parse_quote(i: &mut &'_ str) -> PResult<Expr> {
    preceded("'", s_exp(repeat(0.., parse_expr)))
        .context(StrContext::Label("quote"))
        .map(Expr::Quote)
        .parse_next(i)
}

fn s_exp<'a, O1, F>(inner: F) -> impl Parser<&'a str, O1, ContextError>
where
    F: Parser<&'a str, O1, ContextError>,
{
    delimited(
        '{',
        preceded(multispace0, inner),
        cut_err(preceded(multispace0, '}')).context(StrContext::Label("closing brace")),
    )
}
