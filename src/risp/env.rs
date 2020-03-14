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
        ret.add_builtin("min", builtin_min);
        ret.add_builtin("max", builtin_max);
        ret.add_builtin("gt", builtin_gt);
        ret.add_builtin(">", builtin_gt);
        ret.add_builtin("lt", builtin_lt);
        ret.add_builtin("<", builtin_lt);
        ret.add_builtin("ge", builtin_ge);
        ret.add_builtin(">=", builtin_ge);
        ret.add_builtin("le", builtin_le);
        ret.add_builtin("<=", builtin_le);
        ret.add_builtin("eq", builtin_eq);
        ret.add_builtin("==", builtin_eq);
        ret.add_builtin("ne", builtin_ne);
        ret.add_builtin("!=", builtin_ne);
        ret.add_builtin("if", builtin_if);
        ret.add_builtin("now", builtin_now);
        ret.add_builtin("and", builtin_and);
        ret.add_builtin("or", builtin_or);
        ret.add_builtin("not", builtin_not);

        // add constants
        ret.add_constant("true", val_bool(true));
        ret.add_constant("false", val_bool(false));
        ret.add_constant("nil", val_bool(false));

        ret
    }

    fn add_builtin(&mut self, name: &str, func: Builtin) {
        self.put(name.to_string(), val_builtin(func, name))
    }

    fn add_constant(&mut self, name: &str, val: Box<Val>) {
        self.put(name.to_string(), val);
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