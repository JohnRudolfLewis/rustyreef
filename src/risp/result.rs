use crate::risp::{
    error::RispError,
    val::Val,
};

pub type Result<T> = std::result::Result<T, RispError>;
pub type RispResult = Result<Box<Val>>;