use std::{
    fmt::{Debug},
    hash::Hash,
    marker::Copy,
};

#[derive(Debug)]
pub enum RispError {
    NoChildren,
    NotANumber,
    ParseError(String)
}

impl<T> From<pest::error::Error<T>> for RispError
where
    T: Debug + Ord + Copy + Hash,
{
    fn from(error: pest::error::Error<T>) -> Self {
        RispError::ParseError(format!("{}", error))
    }
}

impl From<std::num::ParseIntError> for RispError {
    fn from(_error: std::num::ParseIntError) -> Self {
        RispError::NotANumber
    }
}