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
#[br(little)]
pub struct Lab {
    research: Param<ResearchMode>,
    accelerator: Param<AcceleratorMode>,
}
#[cfg(feature = "visit")]
impl Visit for Lab {
    fn visit<T: super::visit::Visitor + ?Sized>(&mut self, _visitor: &mut T) {}
}
