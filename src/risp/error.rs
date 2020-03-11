use std::{
    fmt::{self, Debug},
    hash::Hash,
    marker::Copy,
};

#[derive(Debug)]
pub enum RispError {
    ArgumentMismatch,
    NoChildren,
    NotANumber,
    NumArguments(usize, usize),
    ParseError(String),
    UnknownFunction(String),
    WrongType(String, String),
}

impl fmt::Display for RispError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RispError::*;
        match self {
            ArgumentMismatch => write!(f, "Argument mismatch"),
            NoChildren => write!(f, "Val has no children"),
            NotANumber => write!(f, "NaN"),
            NumArguments(expected, received) => write!(
                f,
                "Wrong number of arguments: expected {}, received {}",
                expected, received
            ),
            ParseError(s) => write!(f, "Parse error: {}", s),
            UnknownFunction(s) => write!(f, "Unknown function {}", s),
            WrongType(expected, received) => write!(f, "Wrong type: expected {}, received {}", expected, received),
        }
    }
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

impl From<std::num::ParseFloatError> for RispError {
    fn from(_error: std::num::ParseFloatError) -> Self {
        RispError::NotANumber
    }
}