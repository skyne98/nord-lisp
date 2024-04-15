use std::cell::RefCell;
use std::collections::{HashMap};
use std::rc::Rc;
use crate::ast::{Atom, Expr, Opcode};
use eyre::{ContextCompat, Result};
use walrus::{InstrSeqBuilder, LocalId, ModuleLocals, ValType};
use crate::mir_context::{MirContext, MirSharedContext};

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
            _ => return Err(eyre::eyre!("Unsupported atom: {:?}", atom)),
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
                        let index = locals.get(ident).copied().wrap_err_with(|| format!("Unknown variable: {}", ident))?;
                        bytecode.push(Mir::LocalSet(index));
                    } else {
                        return Err(eyre::eyre!("Invalid assignment target: {:?}", lhs));
                    }
                }
                _ => return Err(eyre::eyre!("Unsupported opcode: {:?}", opcode)),
            }
        }
        Expr::UnaryOp(opcode, expr) => {
            match opcode {
                Opcode::Neg => {
                    bytecode.push(Mir::ConstI64(0));
                    compile_expr(expr, bytecode, locals)?;
                    bytecode.push(Mir::SubI64);
                }
                _ => return Err(eyre::eyre!("Unsupported opcode: {:?}", opcode)),
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
        _ => return Err(eyre::eyre!("Unsupported expression: {:?}", ast)),
    }

    Ok(())
}

pub fn mir_to_wasm(op: &Mir, context: MirSharedContext) -> Result<()> {
    context.borrow_mut().function_body(|mut builder| {
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
                if let Some(local) = context.borrow_mut().get_local(*index) {
                    builder.local_get(local);
                } else {
                    eyre::bail!("Local not found")
                }
            }
            Mir::LocalSet(index) => {
                let index = context.borrow_mut().get_or_add_local(*index, walrus::ValType::I64);
                builder.local_set(index);
            }
            Mir::LocalTee(index) => {
                let index = context.borrow_mut().get_or_add_local(*index, walrus::ValType::I64);
                builder.local_tee(index);
            }
            Mir::Block(ops) => {
                builder.block(ValType::I64, |block| {
                    for op in ops {
                        mir_to_wasm(op, context.clone()).expect("Failed to compile instruction");
                    }
                });
            }
            Mir::IfElse(then_ops, else_ops) => {
                builder.unop(walrus::ir::UnaryOp::I32WrapI64);

                if let Some(else_ops) = else_ops {
                    builder.if_else(ValType::I64, |then| {
                        for op in then_ops {
                            mir_to_wasm(op, context.clone()).expect("Failed to compile instruction");
                        }
                    }, |else_builder| {
                        for op in else_ops {
                            mir_to_wasm(op, context.clone()).expect("Failed to compile instruction");
                        }
                    });
                } else {}
            }
            _ => unimplemented!("Unsupported instruction: {:?}", op),
        }

        Ok(())
    })?;

    Ok(())
}
pub fn to_wat_module(bytecode: &[Mir]) -> Vec<u8> {
    let config = walrus::ModuleConfig::new();
    let module = walrus::Module::with_config(config);
    let context = MirContext::new(module);
    context.borrow_mut().set_new_builder(&[], &[walrus::ValType::I64]);

    for instr in bytecode {
        mir_to_wasm(instr, context.clone()).expect("Failed to compile instruction");
    }

    let function = context.borrow_mut().finish_builder(vec![]).expect("Failed to finish builder");
    context.borrow_mut().export_function("main", function);

    let mut context = context.borrow_mut();
    context.emit_wasm()
}
pub fn to_wasm_module(bytecode: &[Mir]) -> Result<Vec<u8>> {
    let wasm = to_wat_module(bytecode);
    let wat = wasmprinter::print_bytes(&wasm).map_err(|err| eyre::eyre!("Failed to print Wasm: {:#?}", err))?;
    println!("===== Wasm: {} bytes", wasm.len());
    println!("{}", wat);

    Ok(wasm)
}