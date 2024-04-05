use std::collections::{HashMap, HashSet};
use std::io::Read;
use crate::ast::{Atom, Expr, Opcode};
use anyhow::{Context, Result};
use tempfile::NamedTempFile;
use walrus::{InstrSeqBuilder, LocalId, ModuleLocals, ValType};
use wasm_opt::{FileType, OptimizationOptions};

/// Opcodes for the Nord's stack based virtual machine.
#[derive(Debug, Clone)]
pub enum Mir {
    ConstI64(i64),
    AddI64,
    SubI64,
    MulI64,
    DivI64,
    ModI64,
    GreaterThanI64,
    LessThanI64,
    EqualI64,
    NotEqualI64,
    GreaterThanOrEqualI64,
    LessThanOrEqualI64,
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    Block(Vec<Mir>),
    Loop(Vec<Mir>),
    IfElse(Vec<Mir>, Option<Vec<Mir>>),
    Drop,
}

/// Convert an AST node to a sequence of bytecode instructions.
pub fn compile(ast: &Expr) -> Result<Vec<Mir>> {
    let mut bytecode = Vec::new();
    let mut locals = HashMap::new();
    compile_expr(ast, &mut bytecode, &mut locals)?;
    Ok(bytecode)
}
/// Compile an AST expression to bytecode.
fn compile_expr(ast: &Expr, bytecode: &mut Vec<Mir>, locals: &mut HashMap<String, u32>) -> Result<()> {
    match ast {
        Expr::Constant(atom) => match atom {
            Atom::Num(num) => bytecode.push(Mir::ConstI64(*num)),
            Atom::Boolean(b) => bytecode.push(Mir::ConstI64(if *b { 1 } else { 0 })),
            Atom::Identifier(ident) => {
                let index = locals.len() as u32;
                let index = locals.entry(ident.clone()).or_insert(index);
                bytecode.push(Mir::LocalGet(*index));
            }
            _ => return Err(anyhow::anyhow!("Unsupported atom: {:?}", atom)),
        },
        Expr::BinaryOp(lhs, opcode, rhs) => {
            compile_expr(lhs, bytecode, locals)?;
            compile_expr(rhs, bytecode, locals)?;
            match opcode {
                Opcode::Add => bytecode.push(Mir::AddI64),
                Opcode::Mul => bytecode.push(Mir::MulI64),
                Opcode::Sub => bytecode.push(Mir::SubI64),
                Opcode::Div => bytecode.push(Mir::DivI64),
                Opcode::Mod => bytecode.push(Mir::ModI64),
                Opcode::Greater => bytecode.push(Mir::GreaterThanI64),
                Opcode::Less => bytecode.push(Mir::LessThanI64),
                Opcode::Equal => bytecode.push(Mir::EqualI64),
                Opcode::NotEqual => bytecode.push(Mir::NotEqualI64),
                Opcode::GreaterEqual => bytecode.push(Mir::GreaterThanOrEqualI64),
                Opcode::LessEqual => bytecode.push(Mir::LessThanOrEqualI64),
                Opcode::Assign => {
                    if let Expr::Constant(Atom::Identifier(ident)) = &**lhs {
                        let index = locals.get(ident).copied().context("Undefined variable")?;
                        bytecode.push(Mir::LocalSet(index));
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
                    bytecode.push(Mir::ConstI64(0));
                    compile_expr(expr, bytecode, locals)?;
                    bytecode.push(Mir::SubI64);
                }
                _ => return Err(anyhow::anyhow!("Unsupported opcode: {:?}", opcode)),
            }
        }
        Expr::Let(ident, expr) => {
            compile_expr(expr, bytecode, locals)?;
            let index = locals.len() as u32;
            locals.insert(ident.clone(), index);
            bytecode.push(Mir::LocalSet(index));
        }
        Expr::Block(exprs) => {
            let mut block_vec = Vec::new();
            for expr in exprs {
                compile_expr(expr, &mut block_vec, locals)?;
            }
            bytecode.push(Mir::Block(block_vec));
        }
        Expr::IfElse(cond, then_expr, else_expr) => {
            compile_expr(cond, bytecode, locals)?;
            let mut then_vec = Vec::new();
            compile_expr(then_expr, &mut then_vec, locals)?;
            let mut else_vec = None;
            if let Some(else_expr) = else_expr {
                let mut else_vec_some = Vec::new();
                compile_expr(else_expr, &mut else_vec_some, locals)?;
                else_vec = Some(else_vec_some);
            }
            bytecode.push(Mir::IfElse(then_vec, else_vec));
        }
        _ => return Err(anyhow::anyhow!("Unsupported expression: {:?}", ast)),
    }

    Ok(())
}

pub fn mir_to_wasm(op: &Mir, builder: &mut InstrSeqBuilder, locals: &mut ModuleLocals, locals_hash: &mut HashMap<u32, LocalId>) -> Result<()> {
    match op {
        Mir::ConstI64(num) => {
            builder.i64_const(*num);
        }
        Mir::AddI64 => {
            builder.binop(walrus::ir::BinaryOp::I64Add);
        }
        Mir::SubI64 => {
            builder.binop(walrus::ir::BinaryOp::I64Sub);
        }
        Mir::MulI64 => {
            builder.binop(walrus::ir::BinaryOp::I64Mul);
        }
        Mir::DivI64 => {
            builder.binop(walrus::ir::BinaryOp::I64DivS);
        }
        Mir::ModI64 => {
            builder.binop(walrus::ir::BinaryOp::I64RemS);
        }
        Mir::GreaterThanI64 => {
            builder.binop(walrus::ir::BinaryOp::I64GtS);
            builder.unop(walrus::ir::UnaryOp::I64ExtendUI32);
        }
        Mir::LessThanI64 => {
            builder.binop(walrus::ir::BinaryOp::I64LtS);
            builder.unop(walrus::ir::UnaryOp::I64ExtendUI32);
        }
        Mir::EqualI64 => {
            builder.binop(walrus::ir::BinaryOp::I64Eq);
            builder.unop(walrus::ir::UnaryOp::I64ExtendUI32);
        }
        Mir::NotEqualI64 => {
            builder.binop(walrus::ir::BinaryOp::I64Ne);
            builder.unop(walrus::ir::UnaryOp::I64ExtendUI32);
        }
        Mir::GreaterThanOrEqualI64 => {
            builder.binop(walrus::ir::BinaryOp::I64GeS);
            builder.unop(walrus::ir::UnaryOp::I64ExtendUI32);
        }
        Mir::LessThanOrEqualI64 => {
            builder.binop(walrus::ir::BinaryOp::I64LeS);
            builder.unop(walrus::ir::UnaryOp::I64ExtendUI32);
        }
        Mir::LocalGet(index) => {
            if locals_hash.get(index).is_none() {
                let local = locals.add(walrus::ValType::I64);
                locals_hash.insert(*index, local);
            }

            let index = locals_hash.get(index).copied().expect("Local not found");
            builder.local_get(index);
        }
        Mir::LocalSet(index) => {
            if locals_hash.get(index).is_none() {
                let local = locals.add(walrus::ValType::I64);
                locals_hash.insert(*index, local);
            }

            let index = locals_hash.get(index).copied().expect("Local not found");
            builder.local_set(index);
        }
        Mir::LocalTee(index) => {
            if locals_hash.get(index).is_none() {
                let local = locals.add(walrus::ValType::I64);
                locals_hash.insert(*index, local);
            }

            let index = locals_hash.get(index).copied().expect("Local not found");
            builder.local_tee(index);
        }
        Mir::Block(ops) => {
            builder.block(ValType::I64, |block| {
                for op in ops {
                    mir_to_wasm(op, block, locals, locals_hash).expect("Failed to compile instruction");
                }
            });
        }
        Mir::IfElse(then_ops, else_ops) => {
            builder.unop(walrus::ir::UnaryOp::I32WrapI64);
            builder.if_else(ValType::I64, |then| {
                for op in then_ops {
                    mir_to_wasm(op, then, locals, locals_hash).expect("Failed to compile instruction");
                }
            }, |else_builder| {
                // TODO: This is a hack to get the else block to compile.
                let mut locals = ModuleLocals::default();
                let mut locals_hash = HashMap::new();

                if let Some(else_ops) = else_ops {
                    for op in else_ops {
                        mir_to_wasm(op, else_builder, &mut locals, &mut locals_hash).expect("Failed to compile instruction");
                    }
                }
            });
        }
        _ => unimplemented!("Unsupported instruction: {:?}", op),
    }

    Ok(())
}
pub fn to_wat_module(bytecode: &[Mir]) -> Vec<u8> {
    let config = walrus::ModuleConfig::new();
    let mut module = walrus::Module::with_config(config);
    let mut builder = walrus::FunctionBuilder::new(&mut module.types, &[], &[ValType::I64]);
    let mut seq = builder.func_body();

    let locals = &mut module.locals;
    let mut locals_hash = HashMap::new();

    for instr in bytecode {
        mir_to_wasm(instr, &mut seq, locals, &mut locals_hash).expect("Failed to compile instruction");
    }

    drop(seq);
    let function = builder.finish(vec![], &mut module.funcs);
    module.exports.add("main", function);

    module.emit_wasm()
}
pub fn to_wasm_module(bytecode: &[Mir]) -> Result<Vec<u8>> {
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