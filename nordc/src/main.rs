#![feature(try_blocks)]

pub mod ast;
mod lalrpop_lexer;
pub mod lexer;
pub mod bytecode;
mod runtime;

use std::io::{Read, Write};

use anyhow::{Context, Result};
use clap::Parser;
use lalrpop_util::lalrpop_mod;
use logos::Logos;
use tempfile::NamedTempFile;
use wasm_opt::base::ModuleReader;
use wasm_opt::OptimizationOptions;

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

    if cli.std {
        loop {
            // Read a line from stdin
            let mut input = String::new();
            let result: Result<String> = try {
                std::io::stdin().read_line(&mut input)?;
                if input.trim().is_empty() {
                    continue;
                }
                let output = execute(&input, cli.silent)?;
                output
            };
            println!("{:?}", result);
        }
    } else if let Some(script) = cli.execute {
        let output = execute(&script, cli.silent)?;
        println!("{}", output);
    } else if let Some(script_path) = cli.input {
        let input = std::fs::read_to_string(script_path)?;
        let output = execute(&input, cli.silent)?;
        println!("{}", output);
    } else {
        // Interactive mode: read from stdin
        loop {
            let mut input = String::new();
            let result: Result<String> = try {
                std::io::stdin().read_line(&mut input)?;
                if input.trim().is_empty() {
                    continue;
                }
                let output = execute(&input, cli.silent)?;
                output
            };
            if cli.silent == false {
                println!("===== Output:");
                println!("{:?}", result);
            }
        }
    }
    Ok(())
}

/// Executes the script, lexing, parsing, and interpreting the input.
fn execute(
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

    // Parse
    let lexer = lalrpop_lexer::Lexer::new(&input);
    let parser = parser::ExprParser::new();
    let output = parser.parse(lexer);
    match &output {
        Ok(output) => {
            if silent == false {
                println!("===== AST:\n{:#?}", output);
                println!();
            }
        }
        Err(err) => {
            return Err(anyhow::anyhow!("AST Error: {:#?}", err));
        }
    }

    // Get the bytecode
    let ast = output.map_err(|err| anyhow::anyhow!("AST Error: {:#?}", err))?;
    let bytecode = bytecode::compile(&ast)?;
    if silent == false {
        println!("===== Bytecode:\n{:#?}", bytecode);
        println!();
    }

    // Compile to Wasm
    let wasm = bytecode::to_wasm_module(&bytecode).context("Failed to compile to Wasm")?;
    let wasm_opt_input_path = NamedTempFile::new()?.into_temp_path();
    let wasm_opt_output_path = NamedTempFile::new()?.into_temp_path();
    std::fs::write(&wasm_opt_input_path, &wasm)?;
    OptimizationOptions::new_opt_level_4().run(&wasm_opt_input_path, &wasm_opt_output_path)?;
    let mut wasm_opt_output = Vec::new();
    std::fs::File::open(&wasm_opt_output_path)?.read_to_end(&mut wasm_opt_output)?;
    wasm_opt_input_path.close()?;
    wasm_opt_output_path.close()?;
    if silent == false {
        let from_size = wasm.len();
        let to_size = wasm_opt_output.len();
        println!("===== Wasm (before optimization): {} bytes", from_size);
        let wat_before_opt = wasmprinter::print_bytes(&wasm).context("Failed to print Wasm")?;
        println!("{}", wat_before_opt);
        println!("===== Wasm (after optimization): {} bytes", to_size);
        let wat_after_opt = wasmprinter::print_bytes(&wasm_opt_output).context("Failed to print Wasm")?;
        println!("{}", wat_after_opt);
        println!();
    }

    // Run the Wasm
    let mut runtime = runtime::Runtime::new(&wasm_opt_output)?;
    let result = runtime.run::<i64>()?;
    Ok(result.to_string())
}
