use crate::cron_error::CronError;
use crate::time_element::TimeElement;
use std::{num::NonZeroU8, str::FromStr};

pub struct Range<T: TimeElement> {
    start: T,
    end: T,
    step: NonZeroU8
}

pub enum CronToken<T: TimeElement> {
    Literal(T),
    Wildcard,
    Range(Range<T>),
    List(Vec<CronToken<T>>)
}

impl<T: TimeElement> FromStr for CronToken<T> {
    type Err = CronError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let token = s.trim();
        if token.is_empty() { return Err(CronError::InvalidNumber(String::new())); }

        if token.contains(',') {
            let tokens = token
                .split(',')
                .map(|t| match Self::from_str(t)? {
                        CronToken::Wildcard => Err(CronError::Malformed),
                        ct => Ok(ct)
                    })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CronToken::List(tokens))

        } else if token == "*" {
            Ok(CronToken::Wildcard)

        } else if token.contains(['/', '-']) {
            Ok(CronToken::Range(parse_range(token)?))

        } else {
            Ok(CronToken::Literal(parse_num(token)?))
        }
    }
}

fn parse_num<T: TimeElement>(s: &str) -> Result<T, CronError> {
    if s.is_empty() || !s.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(CronError::InvalidNumber(s.to_owned()));
    }

    let num: u8 = s.parse().map_err(|_| CronError::OutOfRange)?; // u8 overflow can happen
    T::try_from(num)
}

fn parse_range<T: TimeElement>(s: &str) -> Result<Range<T>, CronError> {
    let (base, step) = match s.split_once('/') {
        Some((b, st)) => (b, st.parse::<NonZeroU8>().map_err(|_| CronError::InvalidStep)?),
        None => (s, NonZeroU8::MIN),
    };

    let (start, end) = if base == "*" {
        (T::try_from(T::MIN)?, T::try_from(T::MAX)?)
    } else if let Some((a, b)) = base.split_once('-') {
        (parse_num(a)?, parse_num(b)?)
    } else {
        (parse_num(base)?, T::try_from(T::MAX)?)
    };

    if start.get() > end.get() { return Err(CronError::OutOfRange); }
    Ok(Range { start, end, step })
}
