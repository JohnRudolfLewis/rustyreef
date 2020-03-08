use std::collections::HashMap;

use crate::risp::{
    error::RispError,
    result::RispResult,
    val::Val,
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
        ret
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