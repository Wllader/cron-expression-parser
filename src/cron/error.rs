use std::fmt;

#[derive(Debug)]
pub enum Error {
    /// A value lies outside its field's range, or a range ends before it starts.
    OutOfRange,
    /// A step after `/` is zero or not a number.
    InvalidStep,
    /// The expression doesn't have exactly five fields.
    WrongFieldCount,
    /// A field contains something that isn't a number (the offending text is kept).
    InvalidNumber(String),
    /// The day-of-month and month fields never combine into a real date.
    ImpossibleDate,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::OutOfRange => write!(f, "value out of range"),
            Error::InvalidStep => write!(f, "step must be a positive number"),
            Error::WrongFieldCount => write!(f, "expected exactly 5 fields"),
            Error::InvalidNumber(s) => write!(f, "invalid number: {s:?}"),
            Error::ImpossibleDate => write!(f, "day of month never occurs in the given months"),
        }
    }
}

impl std::error::Error for Error {}
