use std::hash::Hash;

use binrw::{BinRead, BinWrite};

pub trait TryFromUserString: Sized {
    fn try_from_user_string(s: &str) -> anyhow::Result<Self>;
}

// These are newtypes for various u16/u32 values in the blueprint. Help make sure we don't misuse
// them and will allow for better localization in the future.

pub trait Nice:
    for<'a> BinWrite<Args<'a> = ()>
    + for<'b> BinRead<Args<'b> = ()>
    + std::fmt::Debug
    + std::fmt::Display
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + Clone
    + Copy
    + Hash
{
}
impl<T> Nice for T where
    T: for<'a> BinWrite<Args<'a> = ()>
        + for<'b> BinRead<Args<'b> = ()>
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + Clone
        + Copy
        + PartialOrd
        + Ord
        + Hash
{
}
