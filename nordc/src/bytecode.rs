use crate::ast::{Atom, Expr, Opcode};
use anyhow::{Context, Result};

/// Opcodes for the Nord's stack based virtual machine.
#[derive(Debug, Clone)]
pub enum Bytecode {
    ConstI64(i64),
    AddI64,
    SubI64,
    MulI64,
    DivI64,
    ModI64,
}

/// Convert an AST node to a sequence of bytecode instructions.
pub fn compile(ast: &Expr) -> Result<Vec<Bytecode>> {
    let mut bytecode = Vec::new();
    compile_expr(ast, &mut bytecode)?;
    Ok(bytecode)
}
/// Compile an AST expression to bytecode.
fn compile_expr(ast: &Expr, bytecode: &mut Vec<Bytecode>) -> Result<()> {
    match ast {
        Expr::Constant(atom) => match atom {
            Atom::Num(num) => bytecode.push(Bytecode::ConstI64(*num)),
            _ => return Err(anyhow::anyhow!("Unsupported atom: {:?}", atom)),
        },
        Expr::BinaryOp(lhs, opcode, rhs) => {
            compile_expr(lhs, bytecode)?;
            compile_expr(rhs, bytecode)?;
            match opcode {
                Opcode::Add => bytecode.push(Bytecode::AddI64),
                Opcode::Mul => bytecode.push(Bytecode::MulI64),
                Opcode::Sub => bytecode.push(Bytecode::SubI64),
                Opcode::Div => bytecode.push(Bytecode::DivI64),
                Opcode::Mod => bytecode.push(Bytecode::ModI64),
                _ => return Err(anyhow::anyhow!("Unsupported opcode: {:?}", opcode)),
            }
        }
        Expr::UnaryOp(opcode, expr) => {
            match opcode {
                Opcode::Neg => {
                    bytecode.push(Bytecode::ConstI64(0));
                    compile_expr(expr, bytecode)?;
                    bytecode.push(Bytecode::SubI64);
                }
                _ => return Err(anyhow::anyhow!("Unsupported opcode: {:?}", opcode)),
            }
        }
        _ => return Err(anyhow::anyhow!("Unsupported expression: {:?}", ast)),
    }

    Ok(())
}

/// Convert a sequence of bytecode instructions to a WAT string.
pub fn to_wat_module(bytecode: &[Bytecode]) -> String {
    let mut wat = String::new();
    wat.push_str("(module\n");
    wat.push_str("  (func $main (result i64)\n");
    for instr in bytecode {
        match instr {
            Bytecode::ConstI64(num) => {
                wat.push_str(&format!("    i64.const {}\n", num));
            }
            Bytecode::AddI64 => {
                wat.push_str("    i64.add\n");
            }
            Bytecode::MulI64 => {
                wat.push_str("    i64.mul\n");
            }
            Bytecode::SubI64 => {
                wat.push_str("    i64.sub\n");
            }
            Bytecode::DivI64 => {
                wat.push_str("    i64.div_s\n");
            }
            Bytecode::ModI64 => {
                wat.push_str("    i64.rem_s\n");
            }
        }
    }
    wat.push_str("  )\n");
    wat.push_str("  (export \"main\" (func $main))\n");
    wat.push_str(")\n");
    wat
}
pub fn to_wasm_module(bytecode: &[Bytecode]) -> Result<Vec<u8>> {
    let wat = to_wat_module(bytecode);
    wat::parse_str(&wat).context("Failed to parse WAT")
}