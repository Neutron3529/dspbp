#[cfg(feature = "visit")]
use super::visit::Visit;
use binrw::{BinRead, BinWrite};
#[cfg(feature = "dump")]
use serde::{Deserialize, Serialize};
// use crate::param::*;

#[cfg_attr(feature = "dump", derive(Serialize, Deserialize))]
#[derive(BinRead, BinWrite, PartialEq, Debug)]
#[br(import(param_count: usize))]
#[br(pre_assert(param_count == 1))]
#[br(little)]
pub struct Sorter {
    pub length: u32,
}
#[cfg(feature = "visit")]
impl Visit for Sorter {
    fn visit<T: super::visit::Visitor + ?Sized>(&mut self, _visitor: &mut T) {}
}
