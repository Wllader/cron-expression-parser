macro_rules! ranged {
    ($name:ident : $type:ty = $min:literal ..= $max:literal) => {
        const _: () = assert!($min <= $max);

        #[derive(Clone, Copy)]
        #[repr(transparent)]
        pub struct $name($type);

        impl $name {
            pub const MIN: $type = $min;
            pub const MAX: $type = $max;

            // # Panics
            // If `val` is outside `MIN..=MAX`. Use `TryFrom` for unchecked input.
            pub const fn new(val: $type) -> Self {
                assert!(val >= Self::MIN && val <= Self::MAX, "Value out of range");
                Self(val)
            }

            pub const fn get(self) -> $type {
                self.0
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&self.get())
                    .finish()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.get(), f)
            }
        }
    };
}

macro_rules! ranged_u8 {
    ($name:ident = $min:literal ..= $max:literal) => { $crate::ranged::ranged!($name:u8 = $min..=$max); };
}

pub(crate) use ranged;
pub(crate) use ranged_u8;
