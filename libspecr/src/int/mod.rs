use crate::*;

mod ops;
mod func;
mod to;
mod step;

pub use to::ToInt;

/// The external Bigint Type, which we use under the hood.
pub(crate) use num_bigint::BigInt as ExtInt;
use num_traits::ToPrimitive;

#[derive(Copy, Clone, Debug, Hash, GcCompat)]
/// Garbage collected big integer that implements `Copy` and supports construction in `const` contexts.
pub struct Int(IntInner);

// IntInner only exists to hide the enum implementation details.
#[derive(Copy, Clone, Debug, Hash, GcCompat)]
enum IntInner {
    Big(GcCow<ExtInt>),
    /// i128 is used to contain u64 and i64.
    Small(i128),
}

impl<T: ~const ToInt> const From<T> for Int {
    fn from(t: T) -> Int {
        t.to_int()
    }
}

impl Int {
    /// The number 0
    pub const ZERO: Int = Int::from(0);
    /// The number 1
    pub const ONE: Int = Int::from(1);

    pub(crate) fn ext(self) -> ExtInt {
        match self.0 {
            IntInner::Big(x) => x.extract(),
            IntInner::Small(x) => x.into(),
        }
    }

    pub(crate) fn wrap(ext: ExtInt) -> Self {
        match ext.to_i128() {
            Some(x) => Self(IntInner::Small(x)),
            None => Self(IntInner::Big(GcCow::new(ext)))
        }
    }
}
