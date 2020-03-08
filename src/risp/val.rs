use crate::risp::{
    error::{RispError},
    result::{Result, RispResult},
};

type ValChildren = Vec<Box<Val>>;

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    Risp(ValChildren),
    Num(i64),
}

// Constructors

pub fn val_risp() -> Box<Val> {
    Box::new(Val::Risp(Vec::new()))
}

pub fn val_num(n: i64) -> Box<Val> {
    Box::new(Val::Num(n))
}

// Manipulating Children

pub fn val_add(v: &mut Val, x: &Val) -> Result<()> {
    match *v {
        Val::Risp(ref mut children) => {
            children.push(Box::new(x.clone()));
        }
        _ => return Err(RispError::NoChildren),
    }
    Ok(())
}

pub fn val_pop(v: &mut Val, i: usize) -> RispResult {
    match *v {
        Val::Risp(ref mut children) => {
            let ret = (&children[i]).clone();
            children.remove(i);
            Ok(ret)
        }
        _ => Err(RispError::NoChildren),
    }
}