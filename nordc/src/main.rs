pub mod ast;
pub mod interpreter;
mod lalrpop_lexer;
pub mod lexer;

use std::io::Read;

use anyhow::Result;
use clap::Parser;
use lalrpop_util::lalrpop_mod;
use logos::Logos;

lalrpop_mod!(pub parser); // synthesized by LALRPOP

/// Your App's CLI options.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom script file to interpret.
    #[clap(short = 'i', long)]
    input: Option<String>,

    /// Executes the script provided directly as an argument.
    #[clap(short = 'e', long)]
    execute: Option<String>,

    /// Silent mode: Only print out the result.
    #[clap(short = 's', long)]
    silent: bool,

    /// Executes from stdin and outputs to stdout.
    #[clap(short = 'd', long)]
    std: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut interpreter = interpreter::Interpreter::new();

    if cli.std {
        loop {
            // Read a line from stdin
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().is_empty() {
                continue;
            }
            let output = execute(&mut interpreter, &input, cli.silent)?;
            println!("{}", output);
        }
    } else if let Some(script) = cli.execute {
        let output = execute(&mut interpreter, &script, cli.silent)?;
        println!("{}", output);
    } else if let Some(script_path) = cli.input {
        let input = std::fs::read_to_string(script_path)?;
        let output = execute(&mut interpreter, &input, cli.silent)?;
        println!("{}", output);
    } else {
        // Interactive mode: read from stdin
        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            if input.trim().is_empty() {
                continue;
            }
            let output = execute(&mut interpreter, &input, cli.silent)?;
            if cli.silent == false {
                println!();
                println!("===== Output:");
                println!("{}", output);
            }
        }
    }
    Ok(())
}

/// Executes the script, lexing, parsing, and interpreting the input.
fn execute(
    interpreter: &mut interpreter::Interpreter,
    input: &str,
    silent: bool,
) -> Result<String> {
    if silent == false {
        // Lex
        let lexer = lexer::Token::lexer(&input);
        print!("===== Tokens:\n");
        for token in lexer {
            println!("{:?}", token);
        }
        println!();
    }

    // Parse and evaluate
    let lexer = lalrpop_lexer::Lexer::new(&input);
    let parser = parser::ExprParser::new();
    let output = parser.parse(lexer);
    match &output {
        Ok(output) => {
            if silent == false {
                println!("===== AST:\n{:#?}", output);
            }
        }
        Err(err) => {
            return Err(anyhow::anyhow!("AST Error: {:#?}", err));
        }
    }

    let output = output.unwrap();
    let result = interpreter.interpret(output);
    Ok(format!("{:#?}", result))
}
