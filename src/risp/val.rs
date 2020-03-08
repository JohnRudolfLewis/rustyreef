use crate::risp::{
    error::{RispError},
    result::{Result, RispResult},
};

type ValChildren = Vec<Box<Val>>;

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    Risp(ValChildren),
    Num(i64),
    Sym(String),
    List(ValChildren),
}

impl Val {
    pub fn len(&self) -> Result<usize> {
        match *self {
            Val::List(ref children) | Val::Risp(ref children) => {
                Ok(children.len())
            }
            _ => Err(RispError::NoChildren),
        }
    }
}

// Constructors

pub fn val_risp() -> Box<Val> {
    Box::new(Val::Risp(Vec::new()))
}

pub fn val_num(n: i64) -> Box<Val> {
    Box::new(Val::Num(n))
}

pub fn val_sym(s: &str) -> Box<Val> {
    Box::new(Val::Sym(s.into()))
}

pub fn val_list() -> Box<Val> {
    Box::new(Val::List(Vec::new()))
}

// Manipulating Children

pub fn val_add(v: &mut Val, x: &Val) -> Result<()> {
    match *v {
        Val::Risp(ref mut children)
        | Val::List(ref mut children) => {
            children.push(Box::new(x.clone()));
        }
        _ => return Err(RispError::NoChildren),
    }
    Ok(())
}

pub fn val_pop(v: &mut Val, i: usize) -> RispResult {
    match *v {
        Val::Risp(ref mut children)
        | Val::List(ref mut children) => {
            let ret = (&children[i]).clone();
            children.remove(i);
            Ok(ret)
        }
        _ => Err(RispError::NoChildren),
    }
}