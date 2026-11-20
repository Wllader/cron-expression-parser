
#[derive(Debug)]
pub enum Error {
    Malformed,
    OutOfRange,
    InvalidStep,
    WrongFieldCount,
    InvalidNumber(String),
    InvalidDate,
    ImpossibleDate
}
