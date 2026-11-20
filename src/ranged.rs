macro_rules! ranged {
    ($name:ident : $type:ty = $min:literal ..= $max:literal) => {
        const _: () = assert!($min <= $max);

        #[derive(Clone, Copy)]
        #[repr(transparent)]
        pub struct $name(pattern_type!($type is $min..=$max));

        impl $name {
            pub const MIN: $type = $min;
            pub const MAX: $type = $max;

            pub const fn new(val: $type) -> Self {
                assert!(val >= Self::MIN && val <= Self::MAX, "Value out of range");
                // SAFETY: Bounds were verified above.
                unsafe { Self(std::mem::transmute::<$type, pattern_type!($type is $min..=$max)>(val)) }
            }

            pub const fn get(self) -> $type {
                // SAFETY: Every inhabitant of `pattern_type!($type is $min..=$max)` is a valid `$type`.
                unsafe { std::mem::transmute(self.0) }
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
