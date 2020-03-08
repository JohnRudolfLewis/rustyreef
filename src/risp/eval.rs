use log::debug;

use crate::risp::{
    env::Env,
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

pub fn eval(e: &mut Env, v: &mut Val) -> RispResult {
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
        _ => {
            debug!("eval: Other: {:?}", v);
            return Ok(Box::new(v.clone()));
        }
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
        let mut parsed = match parse("1") {
            Ok(p) => *p,
            Err(err) => {
                debug!("{}", err);
                return assert!(false, err)
            }, 
        };
        let mut env = Env::new(None);
        let evaled =  match eval(&mut env, &mut parsed) {
            Ok(v) => v,
            Err(err) => {
                debug!("{}", err);
                return assert!(false, err)
            }, 
        };
        assert_eq!(val_num(1), evaled);
    }

    #[test]
    fn eval_symbol() {
        init();
        let mut parsed = match parse("a") {
            Ok(p) => *p,
            Err(err) => {
                debug!("{}", err);
                return assert!(false, err)
            }, 
        };

        let mut env = Env::new(None);
        env.put("a".to_string(), val_num(1));

        let evaled =  match eval(&mut env, &mut parsed) {
            Ok(v) => v,
            Err(err) => {
                debug!("{}", err);
                return assert!(false, err)
            }, 
        };
        assert_eq!(val_num(1), evaled);
    }
}