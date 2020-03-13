use crate::risp::{
    env::Env,
    error::{RispError},
    result::{Result, RispResult},
};
use std::{
    cmp::Ordering,
    fmt
};
use chrono::{NaiveDateTime, NaiveDate, NaiveTime};

type ValChildren = Vec<Box<Val>>;
pub type Builtin = fn(&mut Env, &mut Val) -> RispResult;

#[derive(Clone)]
pub enum ValFun {
    Builtin(String, Builtin),
}

impl fmt::Debug for ValFun {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValFun::Builtin(name, _) => write!(f, "Builtin({})", name),
        }
    }
}

impl fmt::Display for Val {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Val::Bool(b) => write!(formatter, "{}", b),
            Val::Risp(_cells) => write!(formatter, "<toplevel>"),
            Val::Float(f) => write!(formatter, "{}", f),
            Val::Fun(lf) => match lf {
                ValFun::Builtin(name, _) => write!(formatter, "<builtin: {}>", name),
            },
            Val::Num(n) => write!(formatter, "{}", n),
            Val::Sym(s) => write!(formatter, "{}", s),
            Val::List(cell) => write!(formatter, "({})", val_expr_print(cell)),
            Val::Time(t) => write!(formatter, "{}", t),
            Val::Date(d) => write!(formatter, "{}", d),
            Val::DateTime(d) => write!(formatter, "{}", d),
        }
    }
}

fn val_expr_print(cell: &[Box<Val>]) -> String {
    let mut ret = String::new();
    for i in 0..cell.len() {
        ret.push_str(&format!("{}", cell[i]));
        if i < cell.len() - 1 {
            ret.push_str(" ");
        }
    }
    ret
}

impl PartialEq for ValFun {
    fn eq(&self, other: &ValFun) -> bool {
        match self {
            ValFun::Builtin(name, _) => match other {
                ValFun::Builtin(other_name, _) => name == other_name,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    Bool(bool),
    Float(f64),
    Fun(ValFun),
    List(ValChildren),
    Num(i64),
    Risp(ValChildren),
    Sym(String),
    Time(NaiveTime),
    Date(NaiveDate),
    DateTime(NaiveDateTime),
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

    pub fn as_num(&self) -> Result<i64> {
        match *self {
            Val::Num(n) => Ok(n),
            _ => Err(RispError::NotANumber),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match *self {
            Val::Bool(b) => Ok(b),
            _ => Err(RispError::WrongType("bool".to_string(), format!("{}", self))),
        }
    }
}

impl PartialOrd for Val {
   fn partial_cmp(&self, other: &Val) -> Option<Ordering> {
       match self {
           Val::Float(s) => match other {
               Val::Num(o) => s.partial_cmp(&(*o as f64)),
               Val::Float(o) => s.partial_cmp(&o),
               _ => None
           },
           Val::Num(s) => match other {
               Val::Num(o) => Some(s.cmp(o)),
               Val::Float(o) => (*s as f64).partial_cmp(o),
               _ => None
           },
           _ => None
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

pub fn val_builtin(f: Builtin, name: &str) -> Box<Val> {
    Box::new(Val::Fun(ValFun::Builtin(name.to_string(), f)))
}

pub fn val_bool(b: bool) -> Box<Val> {
    Box::new(Val::Bool(b))
}

pub fn val_float(f: f64) -> Box<Val> {
    Box::new(Val::Float(f))
}

pub fn val_time(t: NaiveTime) -> Box<Val> {
    Box::new(Val::Time(t))
}

pub fn val_date(d: NaiveDate) -> Box<Val> {
    Box::new(Val::Date(d))
}

pub fn val_datetime(dt: NaiveDateTime) -> Box<Val> {
    Box::new(Val::DateTime(dt))
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compare_floats() {
        assert!(Val::Float(1.0) > Val::Float(0.0));
    }
}