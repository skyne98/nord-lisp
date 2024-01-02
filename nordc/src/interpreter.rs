// src/interpreter.rs

use crate::ast::{Atom, Expr, Opcode};
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Num(i64),
    Boolean(bool),
    String(String),
    Function(Option<String>, Box<Expr>),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Nothing,
}

impl Value {
    pub fn as_num(&self) -> i64 {
        match self {
            Value::Num(num) => *num,
            Value::Boolean(boolean) => *boolean as i64,
            _ => panic!("Expected number"),
        }
    }

    pub fn as_boolean(&self) -> bool {
        match self {
            Value::Boolean(boolean) => *boolean,
            Value::Num(num) => *num != 0,
            _ => panic!("Expected boolean"),
        }
    }
}

pub struct Interpreter {
    scopes: Vec<HashMap<String, Value>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn interpret(&mut self, ast: Expr) -> Result<Value> {
        let value = match ast {
            Expr::Constant(atom) => match atom {
                Atom::Num(num) => Value::Num(num),
                Atom::Boolean(boolean) => Value::Boolean(boolean),
                Atom::String(string) => Value::String(string),
                Atom::Identifier(name) => {
                    for scope in self.scopes.iter().rev() {
                        if let Some(value) = scope.get(&name) {
                            return Ok(value.clone());
                        }
                    }
                    Err(anyhow::anyhow!("Undefined variable: {}", name))?
                }
                _ => Err(anyhow::anyhow!("Invalid constant"))?,
            },
            Expr::UnaryOp(op, expr) => {
                let expr = self.interpret(*expr)?;
                match op {
                    Opcode::Neg => Value::Num(-expr.as_num()),
                    _ => Err(anyhow::anyhow!("Invalid unary operator"))?,
                }
            }
            Expr::BinaryOp(left, op, right) => {
                let left = self.interpret(*left)?;
                let right = self.interpret(*right)?;
                match op {
                    Opcode::Add => Value::Num(left.as_num() + right.as_num()),
                    Opcode::Sub => Value::Num(left.as_num() - right.as_num()),
                    Opcode::Mul => Value::Num(left.as_num() * right.as_num()),
                    Opcode::Div => Value::Num(left.as_num() / right.as_num()),
                    Opcode::Equal => Value::Boolean(left.as_num() == right.as_num()),
                    Opcode::NotEqual => Value::Boolean(left.as_num() != right.as_num()),
                    Opcode::Less => Value::Boolean(left.as_num() < right.as_num()),
                    Opcode::LessEqual => Value::Boolean(left.as_num() <= right.as_num()),
                    Opcode::Greater => Value::Boolean(left.as_num() > right.as_num()),
                    Opcode::GreaterEqual => Value::Boolean(left.as_num() >= right.as_num()),
                    Opcode::And => Value::Boolean(left.as_boolean() && right.as_boolean()),
                    Opcode::Or => Value::Boolean(left.as_boolean() || right.as_boolean()),
                    _ => Err(anyhow::anyhow!("Invalid binary operator"))?,
                }
            }
            Expr::Let(name, value) => {
                let value = self.interpret(*value)?;
                self.scopes.last_mut().unwrap().insert(name, value.clone());
                value
            }
            Expr::Block(exprs) => {
                self.scopes.push(HashMap::new());
                let mut result = Value::Num(0);
                for expr in exprs {
                    result = self.interpret(expr)?;
                }
                self.scopes.pop();
                result
            }
            Expr::IfElse(cond, if_true, if_false) => {
                let cond = self.interpret(*cond)?;
                if cond.as_boolean() {
                    self.interpret(*if_true)?
                } else if let Some(if_false) = if_false {
                    self.interpret(*if_false)?
                } else {
                    Value::Nothing
                }
            }
            Expr::Index(array, index) => {
                let array = self.interpret(*array)?;
                let index = self.interpret(*index)?;
                match array {
                    Value::Array(array) => array[index.as_num() as usize].clone(),
                    _ => Err(anyhow::anyhow!("Expected array"))?,
                }
            }
            Expr::Member(object, member) => {
                let object = self.interpret(*object)?;
                match object {
                    Value::Object(object) => object[&member].clone(),
                    _ => Err(anyhow::anyhow!("Expected object"))?,
                }
            }
            Expr::Array(exprs) => {
                let mut array = Vec::new();
                for expr in exprs {
                    array.push(self.interpret(expr)?);
                }
                Value::Array(array)
            }
            Expr::Object(exprs) => {
                let mut object = HashMap::new();
                for (name, expr) in exprs {
                    object.insert(name, self.interpret(expr)?);
                }
                Value::Object(object)
            }
            Expr::Lambda(param, expr) => {
                let param = param.map(|param| param.to_string());
                Value::Function(param, expr)
            }
            Expr::Call(function, argument) => {
                let function = self.interpret(*function)?;
                let argument = argument.map(|argument| self.interpret(*argument));
                match function {
                    Value::Function(param, expr) => {
                        self.scopes.push(HashMap::new());
                        if let Some(param) = param {
                            if let Some(Ok(argument)) = argument {
                                self.scopes.last_mut().unwrap().insert(param, argument);
                            }
                        }
                        let result = self.interpret(*expr)?;
                        self.scopes.pop();
                        result
                    }
                    _ => Err(anyhow::anyhow!("Expected function"))?,
                }
            }
            _ => Err(anyhow::anyhow!("Invalid expression"))?,
        };

        Ok(value)
    }
}
