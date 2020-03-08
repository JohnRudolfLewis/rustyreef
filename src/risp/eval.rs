use log::debug;

use crate::risp::{
    result::{RispResult},
    val::*,
};

// Given a slice of boxed Vals, return a single evaluated list
fn eval_cells(cells: &[Box<Val>]) -> RispResult {
    cells.iter().fold(Ok(val_list()), |acc, c| {
        match acc {
            Ok(mut val) => {
                val_add(&mut val, &*eval(&mut c.clone())?)?;
                Ok(val)
            }
            // it's just a Result so we can bubble errors out of the fold
            Err(_) => unreachable!(),
        }
    })
}

pub fn eval(v: &mut Val) -> RispResult {
    let mut args_eval;
    match v {
        Val::Risp(forms) => {
            args_eval = eval_cells(forms)?;
            let forms_len = args_eval.len()?;
            return Ok(val_pop(&mut args_eval, forms_len - 1)?);
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
        let evaled =  match eval(&mut parsed) {
            Ok(v) => v,
            Err(err) => {
                debug!("{}", err);
                return assert!(false, err)
            }, 
        };
        debug!("evaled");
        assert_eq!(val_num(1), evaled);
    }
}