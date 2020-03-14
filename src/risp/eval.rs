use log::debug;
use std::collections::{HashSet};
use std::ops::{Add, Div, Mul, Rem, Sub};
use std::cmp::Ordering;

use chrono::{Duration, NaiveDate, NaiveTime, offset::Local};

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

fn call(e: &mut Env, f: Val, args: &mut Val) -> RispResult {
    debug!("call");
    match f {
        Val::Fun(func) => {
            match func {
                ValFun::Builtin(_name, fp) => {
                    return fp(e, args);
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
            },
            (Val::Num(x_num), Val::Float(y_num)) => {
                $x = val_float((x_num as f64).$op(y_num));
                continue;
            },
            (Val::Float(x_num), Val::Num(y_num)) => {
                $x = val_float(x_num.$op((y_num as f64)));
                continue;
            },
            (Val::Float(x_num), Val::Float(y_num)) => {
                $x = val_float(x_num.$op(y_num));
                continue;
            },
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
                match x.partial_cmp(&y) {
                    Some(o) => {
                        match o {
                            Ordering::Less => {
                                x = x;
                            },
                            Ordering::Greater => {
                                x = y;
                            }
                            _ => return Ok(val_bool(false))
                        }
                    },
                    None => return Err(RispError::ArgumentMismatch)
                }
            },
            "max" => {
                debug!("builtin_op max {} and {}", x, y);
                match x.partial_cmp(&y) {
                    Some(o) => {
                        match o {
                            Ordering::Less => {
                                x = y;
                            },
                            Ordering::Greater => {
                                x = x;
                            }
                            _ => return Ok(val_bool(false))
                        }
                    },
                    None => return Err(RispError::ArgumentMismatch)
                }
            },
            "gt" => {
                debug!("builtin_op gt {} and {}", x, y);
                match x.partial_cmp(&y) {
                    Some(o) => {
                        match o {
                            Ordering::Greater => {
                                x = y;
                            },
                            _ => return Ok(val_bool(false))
                        }
                    },
                    None => return Err(RispError::ArgumentMismatch)
                }
            },
            "lt" => {
                debug!("builtin_op lt {} and {}", x, y);
                match x.partial_cmp(&y) {
                    Some(o) => {
                        match o {
                            Ordering::Less => {
                                x = x;
                            },
                            _ => return Ok(val_bool(false))
                        }
                    },
                    None => return Err(RispError::ArgumentMismatch)
                }
            },
            "ge" => {
                debug!("builtin_op ge {} and {}", x, y);
                match x.partial_cmp(&y) {
                    Some(o) => {
                        match o {
                            Ordering::Greater |
                            Ordering::Equal  => {
                                x = y;
                            },
                            _ => return Ok(val_bool(false))
                        }
                    },
                    None => return Err(RispError::ArgumentMismatch)
                }
            },
            "le" => {
                debug!("builtin_op le {} and {}", x, y);
                match x.partial_cmp(&y) {
                    Some(o) => {
                        match o {
                            Ordering::Less |
                            Ordering::Equal  => {
                                x = y;
                            },
                            _ => return Ok(val_bool(false))
                        }
                    },
                    None => return Err(RispError::ArgumentMismatch)
                }
            },
            "eq" => {
                debug!("builtin_op le {} and {}", x, y);
                match x.partial_cmp(&y) {
                    Some(o) => {
                        match o {
                            Ordering::Equal  => {
                                x = y;
                            },
                            _ => return Ok(val_bool(false))
                        }
                    },
                    None => return Err(RispError::ArgumentMismatch)
                }
            },
            _ => unreachable!(),
        }
    }

    if func == "gt" || func == "lt" || func == "ge" || func == "le" || func == "eq" {
        Ok(val_bool(true))
    } else {
        Ok(x)
    }
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
                "eval: Symbol lookup - retrieved {:?} from key {:?}",
                result, s
            );
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
    } else {
        match *val_peek(&mut args_eval, 0)? {
            Val::Fun(_) => {
                let fp = val_pop(&mut args_eval, 0)?;
                debug!("Calling function {:?} on {:?}", fp, v);
                call(e, *fp, &mut *args_eval)
            },
            _ => {
                debug!("Single-expression");
                eval(e, &mut *val_pop(v, 0)?)
            }
        }
    }
}

pub fn builtin_add(_e: &mut Env, a: &mut Val) -> RispResult {
    builtin_iter_op(a, "add")
}

pub fn builtin_sub(_e: &mut Env, a: &mut Val) -> RispResult {
    builtin_iter_op(a, "sub")
}

pub fn builtin_mul(_e: &mut Env, a: &mut Val) -> RispResult {
    builtin_iter_op(a, "mul")
}

pub fn builtin_div(_e: &mut Env, a: &mut Val) -> RispResult {
    builtin_iter_op(a, "div")
}

pub fn builtin_rem(_e: &mut Env, a: &mut Val) -> RispResult {
    builtin_iter_op(a, "rem")
}

pub fn builtin_min(_e: &mut Env, a: &mut Val) -> RispResult {
    builtin_iter_op(a, "min")
}

pub fn builtin_max(_e: &mut Env, a: &mut Val) -> RispResult {
    builtin_iter_op(a, "max")
}

pub fn builtin_gt(_e: &mut Env, a: &mut Val) -> RispResult {
    builtin_iter_op(a, "gt")
}

pub fn builtin_lt(_e: &mut Env, a: &mut Val) -> RispResult {
    builtin_iter_op(a, "lt")
}

pub fn builtin_ge(_e: &mut Env, a: &mut Val) -> RispResult {
    builtin_iter_op(a, "ge")
}

pub fn builtin_le(_e: &mut Env, a: &mut Val) -> RispResult {
    builtin_iter_op(a, "le")
}

pub fn builtin_eq(_e: &mut Env, a: &mut Val) -> RispResult {
    builtin_iter_op(a, "eq")
}

pub fn builtin_ne(_e: &mut Env, mut a: &mut Val) -> RispResult {
    let mut child_count = match *a {
        Val::List(ref children) => children.len(),
        _ => return Ok(Box::new(a.clone())),
    };

    let mut values = HashSet::new();
    let x = val_pop(&mut a, 0)?;
    values.insert(x.as_num()?);
    while child_count > 1 {
        let y = val_pop(&mut a, 0)?;
        child_count -= 1;
        let y_num = y.as_num()?;
        if !values.contains(&y_num) {
            values.insert(y_num);
        } else {
            return Ok(val_bool(false));
        }
    }
    return Ok(val_bool(true));
}

pub fn builtin_if(e: &mut Env, a: &mut Val) -> RispResult {
    // must have three children
    let child_count = match *a {
        Val::List(ref children) => children.len(),
        _ => return Err(RispError::WrongType("list".to_string(), format!("{:?}", a)))
    };
    if child_count != 3 {
        return Err(RispError::NumArguments(3, child_count));
    }

    // first child must evaluate to bool
    let b = match *val_pop(a, 0)? {
        Val::Bool(b) => b,
        Val::List(cells) => {
            match *eval_cells(e, &cells)? {
                Val::Bool(b) => b,
                _ => return Err(RispError::WrongType("bool".to_string(),format!("{:?}", ""))) // todo improve this error    
            }
        },
        _ => { 
            return Err(RispError::WrongType("bool".to_string(),format!("{:?}", ""))); // todo improve this error
        }
    };
    
    let mut expr_to_eval;
    if b {
        expr_to_eval = val_pop(a, 0)?;
    } else {
        expr_to_eval = val_pop(a, 1)?;
    }

    eval(e, &mut expr_to_eval)
}

pub fn builtin_now(_e: &mut Env, a: &mut Val) -> RispResult {
    // must have zero children
    let child_count = match *a {
        Val::List(ref children) => children.len(),
        _ => return Err(RispError::WrongType("list".to_string(), format!("{:?}", a)))
    };
    if child_count != 0 {
        return Err(RispError::NumArguments(0, child_count));
    }
    let now = Local::now().naive_local();
    debug!("Now {}", now);
    
    return Ok(val_datetime(now));
}

pub fn builtin_and(e: &mut Env, v: &mut Val) -> RispResult {
    // must have more than 1 arg
    let mut arg_count = match *v {
        Val::List(ref children) => children.len(),
        _ => return Err(RispError::WrongType("list".to_string(), format!("{:?}", v)))
    };
    if arg_count < 2 {
        return Err(RispError::NumArguments(2, arg_count));
    }

    // all but the last arg must eval non nil/false
    while arg_count > 1 {
        let res = *eval(e, &mut *val_pop(v,0)?)?;
        match res {
            Val::Bool(b) => {
                if !b {
                    return Ok(val_bool(false));
                }
            },
            _ => { }
        }
        arg_count -= 1;
    }

    // if you got here, all args evaluated true, evaluate the last arg
    let mut last_arg = val_pop(v, 0)?;
    eval(e, &mut last_arg)
}

pub fn builtin_or(e: &mut Env, v: &mut Val) -> RispResult {
    // must have more than 1 arg
    let mut arg_count = match *v {
        Val::List(ref children) => {
            let ret = children.len();
            if ret < 2 {
                return Err(RispError::NumArguments(2, ret));
            }
            ret
        },
        _ => return Err(RispError::WrongType("list".to_string(), format!("{:?}", v)))
    };

    // at least one except the last arg must eval non nil/false
    let mut one_true = false;
    while arg_count > 1 {
        let res = *eval(e, &mut *val_pop(v,0)?)?;
        match res {
            Val::Bool(b) => {
                if b {
                    one_true = true;
                }
            },
            _ => { 
                one_true = true;
            }
        }
        arg_count -= 1;
    }

    if one_true {
        let mut last_arg = val_pop(v, 0)?;
        return eval(e, &mut last_arg);
    } else {
        return Ok(val_bool(false));
    }
}

pub fn builtin_not(e: &mut Env, v: &mut Val) -> RispResult {
    // must have 1 arg
    let mut arg_count = match *v {
        Val::List(ref children) => {
            let ret = children.len();
            if ret != 1 {
                return Err(RispError::NumArguments(1, ret));
            }
            ret
        },
        _ => return Err(RispError::WrongType("list".to_string(), format!("{:?}", v)))
    };

    let res = *eval(e, &mut *val_pop(v,0)?)?;
    
    match res {
        Val::Bool(b) => Ok(val_bool(!b)),
        _ => Ok(val_bool(false))
    }
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
        assert_eval("(max 2 5)", &mut env, val_num(5));
    }

    #[test]
    fn gt_multiple_numbers() {
        init();
        let mut env = Env::new(None);
        assert_eval("(gt 1 0)", &mut env, val_bool(true));
        assert_eval("(gt 0 1)", &mut env, val_bool(false));
        assert_eval("(> 1 0)", &mut env, val_bool(true));
        assert_eval("(> 0 1)", &mut env, val_bool(false));
        assert_eval("(> 3 2 1 0)", &mut env, val_bool(true));
        assert_eval("(> 3 0 1 0)", &mut env, val_bool(false));
    }

    #[test]
    fn lt_multiple_numbers() {
        init();
        let mut env = Env::new(None);
        assert_eval("(< 0 1)", &mut env, val_bool(true));
        assert_eval("(< 0 1 2 3)", &mut env, val_bool(true));
        assert_eval("(lt 1 0)", &mut env, val_bool(false));
        assert_eval("(lt 0 1 0 3)", &mut env, val_bool(false));
    }

    #[test]
    fn ge_multiple_numbers() {
        init();
        let mut env = Env::new(None);
        assert_eval("(ge 1 0)", &mut env, val_bool(true));
        assert_eval("(ge 1 1)", &mut env, val_bool(true));
        assert_eval("(>= 4 4 3 2 1 0)", &mut env, val_bool(true));
        assert_eval("(ge 4 4 0 2 1 0)", &mut env, val_bool(false));
    }

    #[test]
    fn le_multiple_numbers() {
        init();
        let mut env = Env::new(None);
        assert_eval("(le 0 1)", &mut env, val_bool(true));
        assert_eval("(le 1 1)", &mut env, val_bool(true));
        assert_eval("(<= 0 0 1 2 3 4)", &mut env, val_bool(true));
        assert_eval("(le 0 0 1 0 3 4)", &mut env, val_bool(false));
    }

    #[test]
    fn eq_multiple_numbers() {
        init();
        let mut env = Env::new(None);
        assert_eval("(eq 1 1 1)", &mut env, val_bool(true));
        assert_eval("(== 1 1 1)", &mut env, val_bool(true));
        assert_eval("(eq 0 0 1 0 3 4)", &mut env, val_bool(false));
    }

    #[test]
    fn ne_multiple_numbers() {
        init();
        let mut env = Env::new(None);
        assert_eval("(ne 0 1 2)", &mut env, val_bool(true));
        assert_eval("(!= 0 1 2)", &mut env, val_bool(true));
        assert_eval("(ne 0 1 1 2 3)", &mut env, val_bool(false));
    }

    #[test]
    fn if_true() {
        init();
        let mut env = Env::new(None);
        env.put("a".to_string(), val_num(1));
        env.put("b".to_string(), val_num(2));
        assert_eval("(if (< a b) (+ a b) (- a b))", &mut env, val_num(3));
    }

    #[test]
    fn if_false() {
        init();
        let mut env = Env::new(None);
        env.put("a".to_string(), val_num(1));
        env.put("b".to_string(), val_num(2));
        assert_eval("(if (> a b) (+ a b) (- a b))", &mut env, val_num(-1));
    }

    #[test]
    fn add_two_floats() {
        init();
        let mut env = Env::new(None);
        env.put("a".to_string(), val_float(1.2));
        env.put("b".to_string(), val_float(2.3));
        assert_eval("(+ a b)", &mut env, val_float(3.5));
    }

    #[test]
    fn add_float_and_num() {
        init();
        let mut env = Env::new(None);
        env.put("a".to_string(), val_num(3));
        env.put("b".to_string(), val_float(0.1415));
        assert_eval("(+ a b)", &mut env, val_float(3.1415));
        assert_eval("(+ b a)", &mut env, val_float(3.1415));
    }

    #[test]
    fn compare_float_and_num() {
        init();
        let mut env = Env::new(None);
        env.put("a".to_string(), val_num(3));
        env.put("b".to_string(), val_float(0.1415));
        env.put("c".to_string(), val_float(3.0));
        assert_eval("(< a b)", &mut env, val_bool(false));
        assert_eval("(> a b)", &mut env, val_bool(true));
        assert_eval("(>= a c)", &mut env, val_bool(true));
        assert_eval("(<= a c)", &mut env, val_bool(true));
        assert_eval("(== a c)", &mut env, val_bool(true));
        assert_eval("(min a b)", &mut env, val_float(0.1415));
        assert_eval("(max a b)", &mut env, val_num(3));
    }

    #[test]
    fn compare_two_times() {
        init();
        let mut env = Env::new(None);
        env.put("t1".to_string(), val_time(NaiveTime::from_hms(9, 0, 0)));
        env.put("t2".to_string(), val_time(NaiveTime::from_hms(10, 0, 0)));
        assert_eval("(> t1 t2)", &mut env, val_bool(false));
        assert_eval("(< t1 t2)", &mut env, val_bool(true));
        assert_eval("(>= t1 t2)", &mut env, val_bool(false));
        assert_eval("(<= t1 t2)", &mut env, val_bool(true));
        assert_eval("(min t1 t2)", &mut env, val_time(NaiveTime::from_hms(9, 0, 0)));
        assert_eval("(max t1 t2)", &mut env, val_time(NaiveTime::from_hms(10, 0, 0)));
    }

    #[test]
    fn compare_two_dates() {
        init();
        let mut env = Env::new(None);
        env.put("t1".to_string(), val_date(NaiveDate::from_ymd(2020, 3, 12)));
        env.put("t2".to_string(), val_date(NaiveDate::from_ymd(2020, 3, 13)));
        assert_eval("(> t1 t2)", &mut env, val_bool(false));
        assert_eval("(< t1 t2)", &mut env, val_bool(true));
        assert_eval("(>= t1 t2)", &mut env, val_bool(false));
        assert_eval("(<= t1 t2)", &mut env, val_bool(true));
        assert_eval("(min t1 t2)", &mut env, val_date(NaiveDate::from_ymd(2020, 3, 12)));
        assert_eval("(max t1 t2)", &mut env, val_date(NaiveDate::from_ymd(2020, 3, 13)));
    }

    #[test]
    fn compare_two_datetimes() {
        init();
        let mut env = Env::new(None);
        env.put("t1".to_string(), val_datetime(NaiveDate::from_ymd(2020, 3, 12).and_hms(0, 0, 1)));
        env.put("t2".to_string(), val_datetime(NaiveDate::from_ymd(2020, 3, 12).and_hms(0, 0, 2)));
        assert_eval("(> t1 t2)", &mut env, val_bool(false));
        assert_eval("(< t1 t2)", &mut env, val_bool(true));
        assert_eval("(>= t1 t2)", &mut env, val_bool(false));
        assert_eval("(<= t1 t2)", &mut env, val_bool(true));
        assert_eval("(min t1 t2)", &mut env, val_datetime(NaiveDate::from_ymd(2020, 3, 12).and_hms(0, 0, 1)));
        assert_eval("(max t1 t2)", &mut env, val_datetime(NaiveDate::from_ymd(2020, 3, 12).and_hms(0, 0, 2)));
    }

    #[test]
    fn compare_date_and_datetime() {
        init();
        let mut env = Env::new(None);
        env.put("t1".to_string(),  val_datetime(NaiveDate::from_ymd(2020, 3, 12).and_hms(0, 0, 1)));
        env.put("t2".to_string(), val_time(NaiveTime::from_hms(10, 0, 0)));
        assert_eval("(> t1 t2)", &mut env, val_bool(false));
        assert_eval("(> t2 t1)", &mut env, val_bool(true));
    }

    #[test]
    fn can_get_current_datetime_and_compare() {
        init();
        let now = Local::now().naive_local();
        let t1 = now.add(Duration::seconds(-1));
        let t2 = now.add(Duration::seconds(1));
        let mut env = Env::new(None);
        env.put("t1".to_string(), val_datetime(t1));
        env.put("t2".to_string(), val_datetime(t2));
        assert_eval("(> (now) t1)", &mut env, val_bool(true));
        assert_eval("(> (now) t2)", &mut env, val_bool(false));
    }

    #[test]
    fn and_operator() {
        init();
        let mut env = Env::new(None);
        assert_eval("(and (> 1 0) (< 0 1) (== 1 1) (42))", &mut env, val_num(42));
        assert_eval("(and (12) (13) (14) (15))", &mut env, val_num(15));
        assert_eval("(and (1) (0) (nil) (true))", &mut env, val_bool(false));
        assert_eval("(and (1) (0) (false) (true))", &mut env, val_bool(false));
    }

    #[test]
    fn or_operator() {
        init();
        let mut env = Env::new(None);
        assert_eval("(or (> 1 0) (< 0 1) (== 1 1) (42))", &mut env, val_num(42));
        assert_eval("(or (nil) (nil) (1) (42))", &mut env, val_num(42));
    }

    #[test]
    fn not_operator() {
        init();
        let mut env = Env::new(None);
        assert_eval("(not nil)", &mut env, val_bool(true));
        assert_eval("(not 1)", &mut env, val_bool(false));
        assert_eval("(not (> 1 0))", &mut env, val_bool(false));
        assert_eval("(not (< 1 0))", &mut env, val_bool(true));
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