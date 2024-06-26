use logos::Logos;
use std::fmt;

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f\r]+")]
pub enum Token {
    #[token("let")]
    KeywordLet,
    #[token("fn")]
    KeywordFn,
    #[token("if")]
    KeywordIf,
    #[token("then")]
    KeywordThen,
    #[token("else")]
    KeywordElse,
    #[token("return")]
    KeywordReturn,
    #[token("block")]
    KeywordBlock,
    #[token("end")]
    KeywordEnd,
    #[token("loop")]
    KeywordLoop,
    #[token("break")]
    KeywordBreak,
    #[token("continue")]
    KeywordContinue,

    #[token("::")]
    DoubleColon,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("..")]
    DotDot,
    #[token("...")]
    DotDotDot,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("@")]
    At,
    #[token("#")]
    Hash,
    #[token("$")]
    Dollar,

    #[regex("[_a-zA-Z][_0-9a-zA-Z]*", |lex| lex.slice().parse().ok())]
    Identifier(String),
    #[regex(r#"\d+"#, |lex| lex.slice().parse().ok())]
    Integer(i64),
    #[regex("true|false", |lex| lex.slice().parse().ok())]
    Boolean(bool),
    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice().parse().ok())]
    String(String),

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("=")]
    Assign,
    #[token(";")]
    Semicolon,

    #[token("+")]
    OperatorAdd,
    #[token("-")]
    OperatorSub,
    #[token("*")]
    OperatorMul,
    #[token("/")]
    OperatorDiv,
    #[token("%")]
    OperatorMod,

    #[token("==")]
    OperatorEqual,
    #[token("!=")]
    OperatorNotEqual,

    #[token("<")]
    OperatorLess,
    #[token("<=")]
    OperatorLessEqual,
    #[token(">")]
    OperatorGreater,
    #[token(">=")]
    OperatorGreaterEqual,
    #[token("!")]
    OperatorNot,
    #[token("&&")]
    OperatorAnd,
    #[token("||")]
    OperatorOr,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
