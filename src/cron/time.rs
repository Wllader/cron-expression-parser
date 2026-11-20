use crate::cron;
use crate::ranged_u8;
use std::fmt::{Display, Debug};

ranged_u8!(Minute = 0..=59);
ranged_u8!(Hour = 0..=23);
ranged_u8!(DayOfMonth = 1..=31);
ranged_u8!(Month = 1..=12);
ranged_u8!(DayOfWeek = 0..=6);

pub trait TimeElement: Sized + Copy + Display + Debug + TryFrom<u8, Error = cron::Error> {
    const MIN: u8;
    const MAX: u8;
    fn get(self) -> u8;
}

macro_rules! impl_time_element_for {
    ($($type:ident),+ $(,)?) => {$(
        impl TryFrom<u8> for $type {
            type Error = cron::Error;

            fn try_from(v: u8) -> Result<Self, Self::Error> {
                if (Self::MIN..=Self::MAX).contains(&v) {
                    Ok($type::new(v)) // range checked above, so `new` can't panic
                } else {
                    Err(cron::Error::OutOfRange)
                }
            }
        }

        impl TimeElement for $type {
            const MIN: u8 = $type::MIN;
            const MAX: u8 = $type::MAX;

            fn get(self) -> u8 {
                $type::get(self)
            }
        }
    )+};
}

impl_time_element_for!(
    Minute,
    Hour,
    DayOfMonth,
    Month,
    DayOfWeek,
);

impl Month {
    const DAYS_IN_MONTH: [u8; 12] = [
        31, 29, 31, 30, 31, 30,
        31, 31, 30, 31, 30, 31,
    ];

    pub const MASK_31: u64 = Self::months_with(31);
    pub const MASK_30: u64 = Self::months_with(30);

    const fn months_with(days: u8) -> u64 {
        let mut m = 1;
        let mut mask = 0;
        while m <= 12 {
            if Month::raw_max_days(m) == days { mask |= 1 << m; }
            m += 1;
        }
        mask
    }

    pub const fn max_days(&self) -> u8 {
        Self::raw_max_days(self.get())
    }

    pub const fn raw_max_days(month: u8) -> u8 {
        Self::DAYS_IN_MONTH[(month - 1) as usize]
    }
}
