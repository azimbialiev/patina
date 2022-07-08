use core::{fmt};


pub type EncodeResult<T> = Result<T, EncodeError>;

#[derive(Debug, PartialEq, Clone)]
pub enum EncodeError {
    NotEnoughData,
    ExceededMaxLength,
}

impl fmt::Display for EncodeError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        //self.description().fmt(fmt)
        match *self {
            EncodeError::NotEnoughData => write!(fmt, "EncodeError::NotEnoughData"),
            EncodeError::ExceededMaxLength => write!(fmt, "EncodeError::ExceededMaxLength"),
        }
    }
}