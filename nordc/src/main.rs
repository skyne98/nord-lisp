#![feature(try_blocks)]

pub mod ast;
mod lexer_ext;
pub mod lexer;
pub mod mir;
mod runtime;
mod mir_context;
mod mir_var_context;

use std::io::Write;

use color_eyre::eyre::Result;
use clap::Parser;
use eyre::WrapErr;
use lalrpop_util::lalrpop_mod;
use logos::Logos;
use wasmtime::{Config, Engine};
use reedline::{DefaultPrompt, DefaultPromptSegment, EditCommand, Emacs, Keybindings, KeyCode, KeyModifiers, Reedline, ReedlineEvent, Signal};

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
    color_eyre::install()?;
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
        let mut line_editor = Reedline::create();

        let prompt = DefaultPrompt::new(DefaultPromptSegment::Basic("nord".to_string()), DefaultPromptSegment::Empty);

        loop {
            let result: Result<String> = try {
                let signal = line_editor.read_line(&prompt);
                match signal {
                    Ok(Signal::Success(buffer)) => {
                        let output = execute(&buffer, cli.silent)?;
                        output
                    }
                    Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                        break;
                    }
                    x => {
                        eyre::bail!("Unexpected signal: {:?}", x)
                    }
                }
            };
            if cli.silent == false {
                println!("===== Output:");
                println!("{:?}", result);
                println!();
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
    let lexer = lexer_ext::Lexer::new(&input);
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
            return Err(eyre::eyre!("AST Error: {:#?}", err));
        }
    }

    // Get the bytecode
    let ast = output.map_err(|err| eyre::eyre!("AST Error: {:#?}", err))?;
    let bytecode = mir::compile(&ast)?;
    if silent == false {
        println!("===== Bytecode:\n{:#?}", bytecode);
        println!();
    }

    // Compile to Wasm
    let wasm = mir::to_wasm_module(&bytecode).wrap_err_with(|| "Failed to compile to Wasm")?;

    // Run the Wasm
    let config = Config::new();
    let engine = Engine::new(&config).map_err(|err| eyre::eyre!("Failed to create engine: {:#?}", err))?;
    let module = wasmtime::Module::new(&engine, wasm).map_err(|err| eyre::eyre!("Failed to create module: {:#?}", err))?;
    let linker = wasmtime::Linker::new(&engine);
    let mut store = wasmtime::Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &module).map_err(|err| eyre::eyre!("Failed to instantiate module: {:#?}", err))?;

    let func = instance.get_typed_func::<(), i64>(&mut store, "main").map_err(|err| eyre::eyre!("Failed to get function: {:#?}", err))?;
    let result = func.call(&mut store, ()).map_err(|err| eyre::eyre!("Failed to call function: {:#?}", err))?;
    Ok(format!("{:?}", result))
}
