use crate::cron;
use crate::cron::{DayOfMonth, DayOfWeek, Hour, Minute, Month, TimeElement};
use std::str::FromStr;
use std::fmt::{Display, Write};
use chrono::{DateTime as ChronoDt, Utc};

type DateTime = ChronoDt<Utc>;

#[derive(Debug)]
pub struct Expression {
    minute: cron::Token<Minute>,
    hour: cron::Token<Hour>,
    day_of_month: cron::Token<DayOfMonth>,
    month: cron::Token<Month>,
    day_of_week: cron::Token<DayOfWeek>,
}

impl Expression {
    pub fn minute(&self) -> cron::Token<Minute> { self.minute.clone() }
    pub fn hour(&self) -> cron::Token<Hour> { self.hour.clone() }
    pub fn day_of_month(&self) -> cron::Token<DayOfMonth> { self.day_of_month.clone() }
    pub fn month(&self) -> cron::Token<Month> { self.month.clone() }
    pub fn day_of_week(&self) -> cron::Token<DayOfWeek> { self.day_of_week.clone() }

    pub fn json_val_sets(&self) -> String {
        let mut b = String::new();

        writeln!(b, "{{").expect("Writing to a String can't fail");
        writeln!(b, "  \"minute\": {:?},", self.minute.to_raw_vec()).unwrap();
        writeln!(b, "  \"hour\": {:?},", self.hour.to_raw_vec()).unwrap();
        writeln!(b, "  \"dayOfMonth\": {:?},", self.day_of_month.to_raw_vec()).unwrap();
        writeln!(b, "  \"month\": {:?},", self.month.to_raw_vec()).unwrap();
        writeln!(b, "  \"dayOfWeek\": {:?}", self.day_of_week.to_raw_vec()).unwrap();
        writeln!(b, "}}").unwrap();

        b
    }

    pub fn next_since(&self, since: DateTime) -> DateTime {
        todo!()
    }

    pub(super) fn has_possible_date(&self) -> bool {
        let months = self.month.mask();
        let days = self.day_of_month.mask();

        let longest = if months & Month::MASK_31 != 0 { 31 }
            else if months & Month::MASK_30 != 0 { 30 }
            else if months != 0 { 29 }
            else { return false; };

        days != 0 && days.trailing_zeros() <= longest
    }
}

impl Iterator for Expression {
    type Item = DateTime;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {} {} {}",
            self.minute,
            self.hour,
            self.day_of_month,
            self.month,
            self.day_of_week
        )
    }
}

impl FromStr for Expression {
    type Err = cron::Error;

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
            return Err(cron::Error::WrongFieldCount);
        }

        if expr.day_of_week.is_wildcard() && !expr.has_possible_date() {
            return Err(cron::Error::ImpossibleDate)
        }

        Ok(expr)
    }
}

fn next_token<'a, T: TimeElement>(fields: &mut impl Iterator<Item = &'a str>)
    -> Result<cron::Token<T>, cron::Error> {
    fields.next().ok_or(cron::Error::WrongFieldCount)?.parse()
}
