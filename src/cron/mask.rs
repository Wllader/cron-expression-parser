use crate::cron::TimeElement;
use std::marker::PhantomData;

/// A set of values of one time field, stored as bits of a `u64` (bit `n` set means value `n`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Mask<T: TimeElement>(u64, PhantomData<T>);

impl<T: TimeElement> Mask<T> {
    pub const fn from_bits(bits: u64) -> Self { Mask(bits, PhantomData) }
    pub const fn contains(self, v: u8) -> bool { self.0 >> v & 1 == 1 }
    pub fn first(self) -> Option<T> { self.next_in(0) }

    /// The smallest member that is `>= from`.
    pub fn next_in(self, from: u8) -> Option<T> {
        let shifted = self.0 >> from;
        let res = (shifted != 0).then(|| from + shifted.trailing_zeros() as u8)?;
        T::try_from(res).ok()
    }

    pub const fn intersects(self, other: Mask<T>) -> bool { self.0 & other.0 != 0 }
    pub const fn is_empty(self) -> bool { self.0 == 0 }
}
