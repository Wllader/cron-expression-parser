use crate::cron;
use crate::cron::{DayOfMonth, DayOfWeek, Hour, Minute, Month, TimeElement};
use crate::cron::schedule::{Cursor, Schedule};
use std::iter::FusedIterator;
use std::str::FromStr;
use std::fmt::{Display, Write};
use chrono::{DateTime as ChronoDt, Datelike, Timelike, Utc};

type DateTime = ChronoDt<Utc>;

#[derive(Debug)]
pub struct Expression {
    minute: cron::Token<Minute>,
    hour: cron::Token<Hour>,
    day_of_month: cron::Token<DayOfMonth>,
    month: cron::Token<Month>,
    day_of_week: cron::Token<DayOfWeek>,
    schedule: Schedule,
}

impl Expression {
    pub fn minute(&self) -> cron::Token<Minute> { self.minute.clone() }
    pub fn hour(&self) -> cron::Token<Hour> { self.hour.clone() }
    pub fn day_of_month(&self) -> cron::Token<DayOfMonth> { self.day_of_month.clone() }
    pub fn month(&self) -> cron::Token<Month> { self.month.clone() }
    pub fn day_of_week(&self) -> cron::Token<DayOfWeek> { self.day_of_week.clone() }

    pub fn json_val_sets(&self) -> String {
        let sch = &self.schedule;
        let mut b = String::new();

        writeln!(b, "{{").expect("Writing to a String can't fail");
        writeln!(b, "  \"minute\": {:?},", sch.minute.to_vec()).unwrap();
        writeln!(b, "  \"hour\": {:?},", sch.hour.to_vec()).unwrap();
        writeln!(b, "  \"dayOfMonth\": {:?},", sch.monthday.to_vec()).unwrap();
        writeln!(b, "  \"month\": {:?},", sch.month.to_vec()).unwrap();
        writeln!(b, "  \"dayOfWeek\": {:?}", sch.weekday.to_vec()).unwrap();
        writeln!(b, "}}").unwrap();

        b
    }

    const SEARCH_GAP_YEARS: u8 = 8;
    pub fn next_since(&self, since: DateTime) -> Option<DateTime> {
        if since == DateTime::MAX_UTC { return None; }
        let sch = &self.schedule;

        let limit = since.year() + Self::SEARCH_GAP_YEARS as i32;
        let mut cursor = since.naive_utc().next_minute()?;

        while cursor.year() <= limit {
            //? MONTH
            let Some(m) = sch.month.next_in(cursor.month() as u8) else {
                cursor = cursor.next_year()?;
                continue;
            };
            if m.get() > cursor.month() as u8 {
                cursor = cursor.month_start(m)?;
                continue;
            }

            //? DAY
            if !sch.day_matches(cursor) {
                cursor = cursor.next_day()?;
                continue;
            }

            //? HOUR
            let Some(h) = sch.hour.next_in(cursor.hour() as u8) else {
                cursor = cursor.next_day()?;
                continue;
            };
            if h.get() > cursor.hour() as u8 {
                cursor = cursor.hour_start(h)?;
            };

            //? MINUTE
            let Some(m) = sch.minute.next_in(cursor.minute() as u8) else {
                cursor = cursor.next_hour()?;
                continue;
            };
            return Some(cursor.with_minute(m.into())?.and_utc());
        }
        None
    }

    pub fn next(&self) -> Option<DateTime> {
        self.next_since(Utc::now())
    }

    pub fn upcoming_since(&self, since: DateTime) -> Upcoming<'_> {
        Upcoming { expr: self, last: since }
    }

    pub fn upcoming(&self) -> Upcoming<'_> {
        self.upcoming_since(Utc::now())
    }

    pub(super) fn has_possible_date(&self) -> bool {
        let months = self.schedule.month;
        let days = self.schedule.monthday;

        let longest = if months.intersects(Month::MASK_31) { 31 }
            else if months.intersects(Month::MASK_30) { 30 }
            else if !months.is_empty() { 29 }
            else { return false; };

        days.first().is_some_and(|d| d.get() <= longest)
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
        let minute = next_token(&mut tokens)?;
        let hour = next_token(&mut tokens)?;
        let day_of_month = next_token(&mut tokens)?;
        let month = next_token(&mut tokens)?;
        let day_of_week = next_token(&mut tokens)?;

        if tokens.next().is_some() {
            return Err(cron::Error::WrongFieldCount);
        }

        let schedule = Schedule::new(&minute, &hour, &day_of_month, &month, &day_of_week);
        let expr = Self { minute, hour, day_of_month, month, day_of_week, schedule };

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

#[derive(Debug, Clone)]
pub struct Upcoming<'a> {
    expr: &'a Expression,
    last: DateTime
}

impl FusedIterator for Upcoming<'_> {}

impl Iterator for Upcoming<'_> {
    type Item = DateTime;

    fn next(&mut self) -> Option<DateTime> {
        let next = self.expr.next_since(self.last)?;
        self.last = next;
        Some(next)
    }
}
