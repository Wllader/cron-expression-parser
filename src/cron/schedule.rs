use crate::cron::mask::Mask;
use crate::cron::{DayOfMonth, DayOfWeek, Hour, Minute, Month, Token};
use chrono::{Datelike, NaiveDate, NaiveDateTime, TimeDelta, Timelike};

/// The fields' masks, computed once at parse time so searches don't rebuild them.
#[derive(Debug, Clone, Copy)]
pub(super) struct Schedule {
    pub month: Mask<Month>,
    pub monthday: Mask<DayOfMonth>,
    pub weekday: Mask<DayOfWeek>,
    pub hour: Mask<Hour>,
    pub minute: Mask<Minute>,
    both_days_specified: bool,
}

impl Schedule {
    pub fn new(
        minute: &Token<Minute>,
        hour: &Token<Hour>,
        day_of_month: &Token<DayOfMonth>,
        month: &Token<Month>,
        day_of_week: &Token<DayOfWeek>,
    ) -> Self {
        Schedule {
            month: month.mask(),
            monthday: day_of_month.mask(),
            weekday: day_of_week.mask(),
            hour: hour.mask(),
            minute: minute.mask(),
            both_days_specified: !day_of_month.is_wildcard() && !day_of_week.is_wildcard(),
        }
    }

    /// Cron's day rule: when both day fields are restricted, either may match.
    pub fn day_matches(&self, t: NaiveDateTime) -> bool {
        let md = self.monthday.contains(t.day() as u8);
        let wd = self.weekday.contains(t.weekday().num_days_from_sunday() as u8);

        if self.both_days_specified { md || wd } else { md && wd }
    }
}

/// Calendar jumps used by the search. Each one resets every smaller unit.
pub(super) trait Cursor: Sized {
    fn next_year(self) -> Option<Self>;
    fn next_day(self) -> Option<Self>;
    fn next_hour(self) -> Option<Self>;
    fn next_minute(self) -> Option<Self>;

    fn month_start(self, m: Month) -> Option<Self>;
    fn hour_start(self, h: Hour) -> Option<Self>;
}

impl Cursor for NaiveDateTime {
    fn next_year(self) -> Option<Self> {
        NaiveDate::from_ymd_opt(self.year() + 1, 1, 1)?.and_hms_opt(0, 0, 0)
    }
    fn next_day(self) -> Option<Self> {
        self.date().succ_opt()?.and_hms_opt(0, 0, 0)
    }
    fn next_hour(self) -> Option<Self> {
        Some(self.date().and_hms_opt(self.hour(), 0, 0)? + TimeDelta::hours(1))
    }
    fn next_minute(self) -> Option<Self> {
        Some(self.with_second(0)?.with_nanosecond(0)? + TimeDelta::minutes(1))
    }

    fn month_start(self, m: Month) -> Option<Self> {
        NaiveDate::from_ymd_opt(self.year(), m.into(), 1)?.and_hms_opt(0, 0, 0)
    }
    fn hour_start(self, h: Hour) -> Option<Self> {
        self.date().and_hms_opt(h.into(), 0, 0)
    }
}
