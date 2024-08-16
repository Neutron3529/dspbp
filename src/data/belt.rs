#[cfg(feature = "visit")]
use super::visit::Visit;
use crate::param::*;
use binrw::{BinRead, BinWrite};
#[cfg(feature = "dump")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "dump", derive(Serialize, Deserialize))]
#[derive(BinRead, BinWrite, PartialEq, Debug, Clone)]
#[br(import(param_count: usize))]
#[br(pre_assert(param_count == 2))]
pub struct Belt {
    #[br(little)]
    pub label: Param<IconId>,
    #[br(little)]
    pub count: u32,
}
#[cfg(feature = "visit")]
impl Visit for Belt {
    fn visit<T: super::visit::Visitor + ?Sized>(&mut self, _visitor: &mut T) {}
}
