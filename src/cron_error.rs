

pub enum CronError {
    Malformed,
    OutOfRange,
    InvalidStep,
    WrongFieldCount,
    InvalidNumber(String)
}
