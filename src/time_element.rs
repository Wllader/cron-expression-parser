use crate::{cron_error::CronError, ranged_u8};

ranged_u8!(Minute = 0..=59);
ranged_u8!(Hour = 0..=23);
ranged_u8!(DayOfMonth = 1..=31);
ranged_u8!(Month = 1..=12);
ranged_u8!(DayOfWeek = 0..=6);

pub trait TimeElement: Sized + Copy + TryFrom<u8, Error = CronError> {
    const MIN: u8;
    const MAX: u8;
    fn get(self) -> u8;
}

macro_rules! impl_time_element_for {
    ($($type:ident),+ $(,)?) => {$(
        impl TryFrom<u8> for $type {
            type Error = CronError;

            fn try_from(v: u8) -> Result<Self, Self::Error> {
                if (Self::MIN..=Self::MAX).contains(&v) {
                    Ok($type::new(v)) // range checked above, so `new` can't panic
                } else {
                    Err(CronError::OutOfRange)
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
    DayOfWeek
);
