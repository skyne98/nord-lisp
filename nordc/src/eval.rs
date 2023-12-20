use winnow::Parser;

use crate::parser::{parse_expr, Atom, BuiltIn, Expr};

pub fn eval_from_str(src: &str) -> Result<Expr, String> {
    parse_expr
        .parse(src)
        .map_err(|e| e.to_string())
        .and_then(|exp| eval_expression(exp).ok_or_else(|| "Eval failed".to_string()))
}

fn eval_expression(e: Expr) -> Option<Expr> {
    match e {
        Expr::Constant(_) | Expr::Quote(_) => Some(e),
        Expr::If(pred, true_branch) => {
            let reduce_pred = eval_expression(*pred)?;
            if get_bool_from_expr(reduce_pred)? {
                eval_expression(*true_branch)
            } else {
                None
            }
        }
        Expr::IfElse(pred, true_branch, false_branch) => {
            let reduce_pred = eval_expression(*pred)?;
            if get_bool_from_expr(reduce_pred)? {
                eval_expression(*true_branch)
            } else {
                eval_expression(*false_branch)
            }
        }
        Expr::Application(head, tail) => {
            let reduced_head = eval_expression(*head)?;
            let reduced_tail = tail
                .into_iter()
                .map(eval_expression)
                .collect::<Option<Vec<Expr>>>()?;
            if let Expr::Constant(Atom::BuiltIn(bi)) = reduced_head {
                Some(Expr::Constant(match bi {
                    BuiltIn::Plus => Atom::Num(
                        reduced_tail
                            .into_iter()
                            .map(get_num_from_expr)
                            .collect::<Option<Vec<i32>>>()?
                            .into_iter()
                            .sum(),
                    ),
                    BuiltIn::Times => Atom::Num(
                        reduced_tail
                            .into_iter()
                            .map(get_num_from_expr)
                            .collect::<Option<Vec<i32>>>()?
                            .into_iter()
                            .product(),
                    ),
                    BuiltIn::Equal => Atom::Boolean(
                        reduced_tail
                            .iter()
                            .zip(reduced_tail.iter().skip(1))
                            .all(|(a, b)| a == b),
                    ),
                    BuiltIn::Not => {
                        if reduced_tail.len() != 1 {
                            return None;
                        } else {
                            Atom::Boolean(!get_bool_from_expr(
                                reduced_tail.first().cloned().unwrap(),
                            )?)
                        }
                    }
                    BuiltIn::Minus => {
                        Atom::Num(if let Some(first_elem) = reduced_tail.first().cloned() {
                            let fe = get_num_from_expr(first_elem)?;
                            reduced_tail
                                .into_iter()
                                .map(get_num_from_expr)
                                .collect::<Option<Vec<i32>>>()?
                                .into_iter()
                                .skip(1)
                                .fold(fe, |a, b| a - b)
                        } else {
                            Default::default()
                        })
                    }
                    BuiltIn::Divide => {
                        Atom::Num(if let Some(first_elem) = reduced_tail.first().cloned() {
                            let fe = get_num_from_expr(first_elem)?;
                            reduced_tail
                                .into_iter()
                                .map(get_num_from_expr)
                                .collect::<Option<Vec<i32>>>()?
                                .into_iter()
                                .skip(1)
                                .fold(fe, |a, b| a / b)
                        } else {
                            Default::default()
                        })
                    }
                }))
            } else {
                None
            }
        }
    }
}

/// To start we define a couple of helper functions
fn get_num_from_expr(e: Expr) -> Option<i32> {
    if let Expr::Constant(Atom::Num(n)) = e {
        Some(n)
    } else {
        None
    }
}

fn get_bool_from_expr(e: Expr) -> Option<bool> {
    if let Expr::Constant(Atom::Boolean(b)) = e {
        Some(b)
    } else {
        None
    }
}
