use std::collections::HashMap;

use crate::risp::{
    error::RispError,
    eval::*,
    result::RispResult,
    val::*,
};

#[derive(Debug, PartialEq)]
pub struct Env {
    data: HashMap<String, Box<Val>>,
}

impl Env {
    pub fn new(data: Option<HashMap<String, Box<Val>>>) -> Self {
        let mut ret = Self {
            data: data.unwrap_or_default()
        };
        ret.add_builtin("add", builtin_add);
        ret.add_builtin("+", builtin_add);
        ret.add_builtin("sub", builtin_sub);
        ret.add_builtin("-", builtin_sub);
        ret.add_builtin("mul", builtin_mul);
        ret.add_builtin("*", builtin_mul);
        ret.add_builtin("div", builtin_div);
        ret.add_builtin("/", builtin_div);
        ret.add_builtin("rem", builtin_rem);
        ret.add_builtin("%", builtin_rem);
        ret
    }

    fn add_builtin(&mut self, name: &str, func: Builtin) {
        self.put(name.to_string(), val_builtin(func, name))
    }

    pub fn put(&mut self, name: String, val: Box<Val>) {
        let current = self.data.entry(name).or_insert_with(|| val.clone());
        if *val != **current {
            // if it already existed, overwrite it with v
            *current = val;
        }
    }

    pub fn get(&self, k: &str) -> RispResult {
        match self.data.get(k) {
            Some(v) => Ok(v.clone()),
            None => {
                Err(RispError::UnknownFunction(k.to_string()))
            }
        }
    }

}