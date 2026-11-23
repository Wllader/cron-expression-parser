use crate::cron;
use crate::cron::mask::Mask;
use std::{fmt::{Debug, Display}, marker::PhantomData};

pub trait TimeElement: Sized + Copy + Display + Debug + TryFrom<u8, Error = cron::Error> {
    const MIN: u8;
    const MAX: u8;
    fn get(self) -> u8;
}

pub trait Tag: Copy {
    const NAME: &'static str;
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RangedU8<T: Tag, const MIN:u8, const MAX:u8>(u8, PhantomData<T>);

impl<T: Tag, const MIN:u8, const MAX:u8> RangedU8<T, MIN, MAX> {
    pub const fn new(val: u8) -> Self {
        const { assert!(MIN <= MAX) }
        assert!(val >= MIN && val <= MAX, "Value out of range");
        Self(val, PhantomData)
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl<T: Tag, const MIN:u8, const MAX:u8> Debug for RangedU8<T, MIN, MAX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(T::NAME)
            .field(&self.get())
            .finish()
    }
}

impl<T: Tag, const MIN:u8, const MAX:u8> Display for RangedU8<T, MIN, MAX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.get(), f)
    }
}

impl<T: Tag, const MIN:u8, const MAX:u8> TryFrom<u8> for RangedU8<T, MIN, MAX> {
    type Error = cron::Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        if (MIN..=MAX).contains(&v) {
            Ok(Self::new(v)) // range checked above, so `new` can't panic
        } else {
            Err(cron::Error::OutOfRange)
        }
    }
}

impl<T: Tag, const MIN:u8, const MAX:u8>  From<RangedU8<T, MIN, MAX>> for u32 {
    fn from(v: RangedU8<T, MIN, MAX>) -> Self {
        v.get().into()
    }
}


impl<T: Tag, const MIN:u8, const MAX:u8> TimeElement for RangedU8<T, MIN, MAX> {
    const MIN: u8 = MIN;
    const MAX: u8 = MAX;

    fn get(self) -> u8 {
        Self::get(self)
    }
}


macro_rules! ranged_time_elements {
    ($($name:ident = $min:literal ..= $max:literal),+ $(,)?) => {
        #[doc(hidden)]
        pub mod tags {
            $(
                #[derive(Clone, Copy)]
                pub struct $name;
                impl super::Tag for $name {
                    const NAME: &'static str = stringify!($name);
                }
            )+
        }

        $(
            pub type $name = RangedU8<tags::$name, $min, $max>;
        )+
    };
}

ranged_time_elements!(
    Minute = 0..=59,
    Hour = 0..=23,
    DayOfMonth = 1..=31,
    Month = 1..=12,
    DayOfWeek = 0..=6
);

impl Month {
    const DAYS_IN_MONTH: [u8; 12] = [
        31, 29, 31, 30, 31, 30,
        31, 31, 30, 31, 30, 31,
    ];

    pub(crate) const MASK_31: Mask<Month> = Self::months_with(31);
    pub(crate) const MASK_30: Mask<Month> = Self::months_with(30);

    const fn months_with(days: u8) -> Mask<Month> {
        let mut m = 1;
        let mut mask = 0;
        while m <= 12 {
            if Month::raw_max_days(m) == days { mask |= 1 << m; }
            m += 1;
        }

        Mask::from_bits(mask)
    }

    pub const fn max_days(&self) -> u8 {
        Self::raw_max_days(self.get())
    }

    pub const fn raw_max_days(month: u8) -> u8 {
        Self::DAYS_IN_MONTH[(month - 1) as usize]
    }
}
