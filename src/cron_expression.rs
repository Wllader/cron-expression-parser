use crate::cron_token::CronToken;
use crate::cron_error::CronError;
use crate::time_element::{DayOfMonth, DayOfWeek, Hour, Minute, Month, TimeElement};
use std::str::FromStr;


struct CronExpression {
    minute: CronToken<Minute>,
    hour: CronToken<Hour>,
    day_of_month: CronToken<DayOfMonth>,
    month: CronToken<Month>,
    day_of_week: CronToken<DayOfWeek>,
}

impl CronExpression {

}

impl FromStr for CronExpression {
    type Err = CronError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.split_whitespace();
        let expr = Self {
            minute: next_token(&mut tokens)?,
            hour: next_token(&mut tokens)?,
            day_of_month: next_token(&mut tokens)?,
            month: next_token(&mut tokens)?,
            day_of_week: next_token(&mut tokens)?
        };

        if tokens.next().is_some() {
            return Err(CronError::WrongFieldCount);
        }

        Ok(expr)
    }
}

fn next_token<'a, T: TimeElement>(fields: &mut impl Iterator<Item = &'a str>)
    -> Result<CronToken<T>, CronError> {
    fields.next().ok_or(CronError::WrongFieldCount)?.parse()
}
