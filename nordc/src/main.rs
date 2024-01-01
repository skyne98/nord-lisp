pub mod ast;
pub mod interpreter;
mod lalrpop_lexer;
pub mod lexer;

use anyhow::Result;
use lalrpop_util::lalrpop_mod;
use logos::Logos;

lalrpop_mod!(pub parser); // synthesized by LALRPOP

fn main() -> Result<()> {
    let mut interpreter = interpreter::Interpreter::new();

    // Simple eval loop
    loop {
        // Read
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if input.trim().len() == 0 {
            continue;
        }

        // Lex
        let lexer = lexer::Token::lexer(&input);
        print!("===== Tokens:\n");
        for token in lexer {
            println!("{:?}", token);
        }
        println!();

        // Eval
        let lexer = lalrpop_lexer::Lexer::new(&input);
        let parser = parser::ExprParser::new();
        let output = parser.parse(lexer);
        match &output {
            Ok(output) => println!("===== AST:\n{:#?}", output),
            Err(err) => {
                println!("===== AST Error:\n{:#?}", err);
                continue;
            }
        }

        let output = output.unwrap();
        let output = interpreter.interpret(output);
        println!("===== Output: {:#?}", output);
    }
}
