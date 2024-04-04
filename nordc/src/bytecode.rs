use std::collections::{HashMap, HashSet};
use std::io::Read;
use crate::ast::{Atom, Expr, Opcode};
use anyhow::{Context, Result};
use tempfile::NamedTempFile;
use walrus::ValType;
use wasm_opt::{FileType, OptimizationOptions};

/// Opcodes for the Nord's stack based virtual machine.
#[derive(Debug, Clone)]
pub enum Bytecode {
    ConstI64(i64),
    AddI64,
    SubI64,
    MulI64,
    DivI64,
    ModI64,
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    End,
    Drop,
}

/// Convert an AST node to a sequence of bytecode instructions.
pub fn compile(ast: &Expr) -> Result<Vec<Bytecode>> {
    let mut bytecode = Vec::new();
    let mut locals = HashMap::new();
    compile_expr(ast, &mut bytecode, &mut locals)?;
    Ok(bytecode)
}
/// Compile an AST expression to bytecode.
fn compile_expr(ast: &Expr, bytecode: &mut Vec<Bytecode>, locals: &mut HashMap<String, u32>) -> Result<()> {
    match ast {
        Expr::Constant(atom) => match atom {
            Atom::Num(num) => bytecode.push(Bytecode::ConstI64(*num)),
            Atom::Identifier(ident) => {
                let index = locals.len() as u32;
                let index = locals.entry(ident.clone()).or_insert(index);
                bytecode.push(Bytecode::LocalGet(*index));
            }
            _ => return Err(anyhow::anyhow!("Unsupported atom: {:?}", atom)),
        },
        Expr::BinaryOp(lhs, opcode, rhs) => {
            compile_expr(lhs, bytecode, locals)?;
            compile_expr(rhs, bytecode, locals)?;
            match opcode {
                Opcode::Add => bytecode.push(Bytecode::AddI64),
                Opcode::Mul => bytecode.push(Bytecode::MulI64),
                Opcode::Sub => bytecode.push(Bytecode::SubI64),
                Opcode::Div => bytecode.push(Bytecode::DivI64),
                Opcode::Mod => bytecode.push(Bytecode::ModI64),
                Opcode::Assign => {
                    if let Expr::Constant(Atom::Identifier(ident)) = &**lhs {
                        let index = locals.get(ident).copied().context("Undefined variable")?;
                        bytecode.push(Bytecode::LocalSet(index));
                    } else {
                        return Err(anyhow::anyhow!("Invalid assignment target: {:?}", lhs));
                    }
                }
                _ => return Err(anyhow::anyhow!("Unsupported opcode: {:?}", opcode)),
            }
        }
        Expr::UnaryOp(opcode, expr) => {
            match opcode {
                Opcode::Neg => {
                    bytecode.push(Bytecode::ConstI64(0));
                    compile_expr(expr, bytecode, locals)?;
                    bytecode.push(Bytecode::SubI64);
                }
                _ => return Err(anyhow::anyhow!("Unsupported opcode: {:?}", opcode)),
            }
        }
        Expr::Let(ident, expr) => {
            compile_expr(expr, bytecode, locals)?;
            let index = locals.len() as u32;
            locals.insert(ident.clone(), index);
            bytecode.push(Bytecode::LocalSet(index));
        }
        Expr::Block(exprs) => {
            for expr in exprs {
                compile_expr(expr, bytecode, locals)?;
            }
        }
        _ => return Err(anyhow::anyhow!("Unsupported expression: {:?}", ast)),
    }

    Ok(())
}

/// Convert a sequence of bytecode instructions to a WAT string.
pub fn to_wat_module(bytecode: &[Bytecode]) -> Vec<u8> {
    let config = walrus::ModuleConfig::new();
    let mut module = walrus::Module::with_config(config);
    let mut builder = walrus::FunctionBuilder::new(&mut module.types, &[], &[ValType::I64]);
    let mut seq = builder.func_body();

    let locals = &mut module.locals;
    let mut locals_hash = HashMap::new();

    for instr in bytecode {
        match instr {
            Bytecode::ConstI64(num) => {
                seq.i64_const(*num);
            }
            Bytecode::AddI64 => {
                seq.binop(walrus::ir::BinaryOp::I64Add);
            }
            Bytecode::SubI64 => {
                seq.binop(walrus::ir::BinaryOp::I64Sub);
            }
            Bytecode::MulI64 => {
                seq.binop(walrus::ir::BinaryOp::I64Mul);
            }
            Bytecode::DivI64 => {
                seq.binop(walrus::ir::BinaryOp::I64DivS);
            }
            Bytecode::ModI64 => {
                seq.binop(walrus::ir::BinaryOp::I64RemS);
            }
            Bytecode::LocalGet(index) => {
                if locals_hash.get(index).is_none() {
                    let local = locals.add(walrus::ValType::I64);
                    locals_hash.insert(*index, local);
                }

                let index = locals_hash.get(index).copied().expect("Local not found");
                seq.local_get(index);
            }
            Bytecode::LocalSet(index) => {
                if locals_hash.get(index).is_none() {
                    let local = locals.add(walrus::ValType::I64);
                    locals_hash.insert(*index, local);
                }

                let index = locals_hash.get(index).copied().expect("Local not found");
                seq.local_set(index);
            }
            Bytecode::LocalTee(index) => {
                if locals_hash.get(index).is_none() {
                    let local = locals.add(walrus::ValType::I64);
                    locals_hash.insert(*index, local);
                }

                let index = locals_hash.get(index).copied().expect("Local not found");
                seq.local_tee(index);
            }
            _ => unimplemented!("Unsupported instruction: {:?}", instr),
        }
    }

    drop(seq);
    let function = builder.finish(vec![], &mut module.funcs);
    module.exports.add("main", function);

    module.emit_wasm()
}
pub fn to_wasm_module(bytecode: &[Bytecode]) -> Result<Vec<u8>> {
    let wasm = to_wat_module(bytecode);
    let wat = wasmprinter::print_bytes(&wasm).context("Failed to print Wasm")?;
    println!("===== Wasm (before optimization): {} bytes", wasm.len());
    println!("{}", wat);
    println!();

    let wasm_opt_input_path = NamedTempFile::new()?.into_temp_path();
    let wasm_opt_output_path = NamedTempFile::new()?.into_temp_path();
    std::fs::write(&wasm_opt_input_path, &wasm)?;
    OptimizationOptions::new_opt_level_4().reader_file_type(FileType::Any).run(&wasm_opt_input_path, &wasm_opt_output_path)?;
    let mut wasm_opt_output = Vec::new();
    std::fs::File::open(&wasm_opt_output_path)?.read_to_end(&mut wasm_opt_output)?;
    wasm_opt_input_path.close()?;
    wasm_opt_output_path.close()?;
    let to_size = wasm_opt_output.len();
    println!("===== Wasm (after optimization): {} bytes", to_size);
    let wat_after_opt = wasmprinter::print_bytes(&wasm_opt_output).context("Failed to print Wasm")?;
    println!("{}", wat_after_opt);
    println!();

    Ok(wasm_opt_output)
}