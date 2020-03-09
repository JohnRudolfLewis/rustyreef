use log::debug;
use std::ops::{Add, Div, Mul, Rem, Sub};

use crate::risp::{
    env::Env,
    error::RispError,
    result::{RispResult},
    val::*,
};

// Given a slice of boxed Vals, return a single evaluated list
fn eval_cells(e: &mut Env, cells: &[Box<Val>]) -> RispResult {
    cells.iter().fold(Ok(val_list()), |acc, c| {
        match acc {
            Ok(mut val) => {
                val_add(&mut val, &*eval(e, &mut c.clone())?)?;
                Ok(val)
            }
            // it's just a Result so we can bubble errors out of the fold
            Err(_) => unreachable!(),
        }
    })
}

fn call(_e: &mut Env, f: Val, args: &mut Val) -> RispResult {
    match f {
        Val::Fun(func) => {
            match func {
                ValFun::Builtin(_name, fp) => {
                    return fp(args);
                }
            }
        },
        _ => Err(RispError::WrongType("Function".to_string(), format!("{:?}", f))),
    }
}

// macro to shorten code for applying a binary operation to two Lvals
macro_rules! apply_binop {
    ( $op:ident, $x:ident, $y:ident ) => {
        match (*$x, *$y) {
            (Val::Num(x_num), Val::Num(y_num)) => {
                $x = val_num(x_num.$op(y_num));
                continue;
            }
            _ => return Err(RispError::NotANumber),
        }
    };
}

// apply a binary operation operation to a list of arguments in succession
fn builtin_iter_op(mut v: &mut Val, func: &str) -> RispResult {
    let mut child_count = match *v {
        Val::List(ref children) => children.len(),
        _ => return Ok(Box::new(v.clone())),
    };

    let mut x = val_pop(&mut v, 0)?;

    // If no args given and we're doing subtraction, perform unary negation
    if func == "sub" && child_count == 1 {
        debug!("builtin_op: Unary negation on {}", x);
        let x_num = x.as_num()?;
        return Ok(val_num(-x_num));
    }

    // consume the children until empty and operate on x
    while child_count > 1 {
        let y = val_pop(&mut v, 0)?;
        child_count -= 1;
        match func {
            "add" => {
                debug!("builtin_op add {} and {}", x, y);
                apply_binop!(add, x, y);
            },
            "sub" => {
                debug!("builtin_op sub {} and {}", x, y);
                apply_binop!(sub, x, y);
            },
            "mul" => {
                debug!("builtin_op mul {} and {}", x, y);
                apply_binop!(mul, x, y);
            },
            "div" => {
                debug!("builtin_op mul {} and {}", x, y);
                apply_binop!(div, x, y);
            },
            "rem" => {
                debug!("builtin_op rem {} and {}", x, y);
                apply_binop!(rem, x, y);
            },
            "min" => {
                debug!("builtin_op min {} and {}", x, y);
                let x_num = x.as_num()?;
                let y_num = y.as_num()?;
                if x_num < y_num {
                    x = val_num(x_num);
                } else {
                    x = val_num(y_num);
                };
            },
            "max" => {
                debug!("builtin_op max {} and {}", x, y);
                let x_num = x.as_num()?;
                let y_num = y.as_num()?;
                if x_num > y_num {
                    x = val_num(x_num);
                } else {
                    x = val_num(y_num);
                };
            }
            _ => unreachable!(),
        }
    }

    Ok(x)
}

pub fn eval(e: &mut Env, v: &mut Val) -> RispResult {
    let child_count;
    let mut args_eval;
    match v {
        Val::Risp(forms) => {
            args_eval = eval_cells(e, forms)?;
            let forms_len = args_eval.len()?;
            return Ok(val_pop(&mut args_eval, forms_len - 1)?);
        }
        Val::Sym(s) => {
            let result = e.get(&s)?;
            debug!(
                "lval_eval: Symbol lookup - retrieved {:?} from key {:?}",
                result, s
            );
            // The environment stores Lvals ready to go, we're done
            return Ok(result);
        }
        Val::List(ref mut cells) => {
            debug!("eval: List, evaluating children");
            child_count = cells.len();
            args_eval = eval_cells(e, cells)?;
        }
        _ => {
            debug!("eval: Other: {:?}", v);
            return Ok(Box::new(v.clone()));
        }
    }
    if child_count == 0 {
        Ok(Box::new(v.clone()))
    } else if child_count == 1 {
        debug!("Single-expression");
        eval(e, &mut *val_pop(v, 0)?)
    } else {
        let fp = val_pop(&mut args_eval, 0)?;
        debug!("Calling function {:?} on {:?}", fp, v);
        call(e, *fp, &mut *args_eval)
    }
}

pub fn builtin_add(a: &mut Val) -> RispResult {
    builtin_iter_op(a, "add")
}

pub fn builtin_sub(a: &mut Val) -> RispResult {
    builtin_iter_op(a, "sub")
}

pub fn builtin_mul(a: &mut Val) -> RispResult {
    builtin_iter_op(a, "mul")
}

pub fn builtin_div(a: &mut Val) -> RispResult {
    builtin_iter_op(a, "div")
}

pub fn builtin_rem(a: &mut Val) -> RispResult {
    builtin_iter_op(a, "rem")
}

pub fn builtin_min(a: &mut Val) -> RispResult {
    builtin_iter_op(a, "min")
}

pub fn builtin_max(a: &mut Val) -> RispResult {
    builtin_iter_op(a, "max")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::risp::{
        parse::parse,
    };

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn eval_single_number() {
        init();
        let mut env = Env::new(None);
        assert_eval("1", &mut env, val_num(1));
    }

    #[test]
    fn eval_symbol() {
        init();
        let mut env = Env::new(None);
        env.put("a".to_string(), val_num(1));
        assert_eval("a", &mut env, val_num(1));
    }

    #[test]
    fn add_two_numbers() {
        init();
        let mut env = Env::new(None);
        assert_eval("(add 1 1)", &mut env, val_num(2));
    }

    #[test]
    fn add_two_numbers_alias() {
        init();
        let mut env = Env::new(None);
        assert_eval("(+ 1 1)", &mut env, val_num(2));
    }

    #[test]
    fn add_three_numbers() {
        init();
        let mut env = Env::new(None);
        assert_eval("(add 1 1 1)", &mut env, val_num(3));
    }

    #[test]
    fn add_numbers_and_symbols() {
        init();
        let mut env = Env::new(None);
        env.put("a".to_string(), val_num(1));
        assert_eval("(add 1 1 a)", &mut env, val_num(3));
    }

    #[test]
    fn subtract_one_number() {
        init();
        let mut env = Env::new(None);
        assert_eval("(sub 1)", &mut env, val_num(-1));
    }

    #[test]
    fn subtract_two_numbers() {
        init();
        let mut env = Env::new(None);
        assert_eval("(sub 1 1)", &mut env, val_num(0));
    }

    #[test]
    fn subtract_two_numbers_alias() {
        init();
        let mut env = Env::new(None);
        assert_eval("(- 1 1)", &mut env, val_num(0));
    }

    #[test]
    fn multiply_two_numbers() {
        init();
        let mut env = Env::new(None);
        assert_eval("(mul 2 2)", &mut env, val_num(4));
    }

    #[test]
    fn multiply_two_numbers_alias() {
        init();
        let mut env = Env::new(None);
        assert_eval("(* 2 2)", &mut env, val_num(4));
    }

    #[test]
    fn divide_two_numbers() {
        init();
        let mut env = Env::new(None);
        assert_eval("(div 4 2)", &mut env, val_num(2));
    }

    #[test]
    fn divide_two_numbers_alias() {
        init();
        let mut env = Env::new(None);
        assert_eval("(/ 4 2)", &mut env, val_num(2));
    }

    #[test]
    fn rem_two_numbers() {
        init();
        let mut env = Env::new(None);
        assert_eval("(rem 5 2)", &mut env, val_num(1));
        assert_eval("(% 5 2)", &mut env, val_num(1));
    }

    #[test]
    fn min_two_numbers() {
        init();
        let mut env = Env::new(None);
        assert_eval("(min 5 2)", &mut env, val_num(2));
    }

    #[test]
    fn max_two_numbers() {
        init();
        let mut env = Env::new(None);
        assert_eval("(max 5 2)", &mut env, val_num(5));
    }

    fn assert_eval(s: &str, env: &mut Env, v: Box<Val>) {
        let mut parsed = match parse(s) {
            Ok(p) => *p,
            Err(err) => {
                debug!("{}", err);
                return assert!(false, err)
            }, 
        };
        let evaled =  match eval(env, &mut parsed) {
            Ok(v) => v,
            Err(err) => {
                debug!("{}", err);
                return assert!(false, err)
            }, 
        };
        assert_eq!(v, evaled);
    }
}