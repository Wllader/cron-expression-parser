use crate::cron::{self, DayOfMonth, DayOfWeek, Expression, Hour, Minute, Month, TimeElement, Token};
use crate::cron::mask::Mask;

fn tok<T: TimeElement>(s: &str) -> Result<Token<T>, cron::Error> {
    s.parse()
}

fn raw<T: TimeElement>(s: &str) -> Vec<u8> {
    tok::<T>(s).unwrap().mask().to_vec()
}

mod time;
mod token;
mod expression;
mod schedule;
