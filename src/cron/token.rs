use crate::cron;
use crate::cron::TimeElement;
use crate::cron::mask::Mask;
use std::{num::NonZeroU8, str::FromStr};
use std::fmt::{Display, Debug};

#[derive(Debug, Clone)]
pub struct Range<T: TimeElement> {
    start: T,
    end: T,
    step: NonZeroU8
}

impl<T: TimeElement> Range<T> {
    pub fn start(&self) -> u8 { self.start.get() }
    pub fn end(&self) -> u8 { self.end.get() }
    pub fn step(&self) -> u8 { self.step.get() }
    pub fn values(&self) -> impl Iterator<Item = u8> {
        (self.start()..=self.end()).step_by(self.step() as usize)
    }
}

#[derive(Clone)]
pub enum Token<T: TimeElement> {
    Literal(T),
    Wildcard,
    Range(Range<T>),
    List(Vec<Token<T>>)
}

impl<T: TimeElement> Token<T> {
    pub fn is_wildcard(&self) -> bool {
        matches!(self, Token::Wildcard)
    }

    pub(crate) fn mask(&self) -> Mask<T> {
        const { assert!(T::MAX < 64, "Masking is bound by bit pattern of u64") };
        let val = match self {
            Token::Literal(v) => 1u64 << v.get(),
            Token::Wildcard => between(T::MIN, T::MAX),
            Token::Range(r) if r.step() == 1 => between(r.start(), r.end()),
            Token::Range(r) => r.values().fold(0, |m, v| m | 1u64 << v),
            Token::List(tokens) =>
                tokens.iter().fold(0, |m, t| m | t.mask().raw_bits())
        };

        Mask::from_bits(val)
    }
}

fn between(start:u8, end:u8) -> u64 {
    assert!(start <= end && end < 64);
    (u64::MAX >> (63 - end)) & (u64::MAX << start)
}

impl<T: TimeElement> Display for Token<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Literal(v) => write!(f, "{}", v.get()),
            Token::Wildcard => write!(f, "*"),
            Token::Range(r) => {
                if r.start() == T::MIN && r.end() == T::MAX {
                    write!(f, "*")?;
                } else {
                    write!(f, "{}-{}", r.start, r.end)?;
                };

                if r.step.get() != 1 {
                    write!(f, "/{}", r.step.get())?;
                };

                Ok(())
            },
            Token::List(items) => {
                for (i, t) in items.iter().enumerate() {
                    if i > 0 { write!(f, ",")?; }
                    write!(f, "{t}")?;
                };
                Ok(())
            },
        }
    }
}

impl<T: TimeElement> Debug for Token<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal(arg0) => write!(f, "{}", arg0.get()),
            Self::Wildcard => write!(f, "Wildcard"),
            Self::Range(arg0) => f.debug_struct("Range")
                .field("Start", &arg0.start())
                .field("End", &arg0.end())
                .field("Step", &arg0.step())
                .finish(),
            Self::List(arg0) => f.debug_tuple("List").field(arg0).finish(),
        }
    }
}

impl<T: TimeElement> FromStr for Token<T> {
    type Err = cron::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let token = s.trim();
        if token.is_empty() { return Err(cron::Error::InvalidNumber(String::new())); }

        if token.contains(',') {
            let tokens = token
                .split(',')
                .map(Self::from_str)
                .collect::<Result<Vec<_>, _>>()?;

            if tokens.iter().any(|t| t.is_wildcard()) {
                Ok(Token::Wildcard)
            } else {
                Ok(Token::List(tokens))
            }

        } else if token == "*" {
            Ok(Token::Wildcard)

        } else if token.contains(['/', '-']) {
            let r: Range<T> = parse_range(token)?;
            if r.start() == r.end() {
                Ok(Token::Literal(r.start))
            } else if r.step() == 1 && r.start() == T::MIN && r.end() == T::MAX {
                Ok(Token::Wildcard)
            } else {
                Ok(Token::Range(r))
            }

        } else {
            Ok(Token::Literal(parse_num(token)?))
        }
    }
}

fn parse_num<T: TimeElement>(s: &str) -> Result<T, cron::Error> {
    if s.is_empty() || !s.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(cron::Error::InvalidNumber(s.to_owned()));
    }

    let num: u8 = s.parse().map_err(|_| cron::Error::OutOfRange)?; // u8 overflow can happen
    T::try_from(num)
}

fn parse_range<T: TimeElement>(s: &str) -> Result<Range<T>, cron::Error> {
    let (base, step) = match s.split_once('/') {
        Some((b, st)) => {
            if st.is_empty() || !st.chars().all(|ch| ch.is_ascii_digit()) {
                return Err(cron::Error::InvalidStep)
            }
            (b, st.parse::<NonZeroU8>().map_err(|_| cron::Error::InvalidStep)?)
        }
        None => (s, NonZeroU8::MIN),
    };

    let (start, end) = if base == "*" {
        (T::try_from(T::MIN)?, T::try_from(T::MAX)?)
    } else if let Some((a, b)) = base.split_once('-') {
        (parse_num(a)?, parse_num(b)?)
    } else {
        (parse_num(base)?, T::try_from(T::MAX)?)
    };

    if start.get() > end.get() { return Err(cron::Error::OutOfRange); }
    Ok(Range { start, end, step })
}
