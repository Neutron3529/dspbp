#[cfg(feature = "visit")]
use super::visit::Visit;
use crate::param::*;
use binrw::{BinRead, BinWrite};
#[cfg(feature = "dump")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "dump", derive(Serialize, Deserialize))]
#[derive(BinRead, BinWrite, PartialEq, Debug)]
#[br(import(param_count: usize))]
#[br(pre_assert(param_count == 2))]
#[br(little)]
pub struct Lab {
    research: Param<ResearchMode>,
    accelerator: Param<AcceleratorMode>,
}
struct ResearchMode;
impl Name for ResearchMode {
    fn name(i: i32) -> Option<&'static str> {
        Some(match i {
            0 => "未选择",
            1 => "矩阵合成",
            2 => "科研模式",
            _ => return None,
        })
    }
}
struct AcceleratorMode;
impl Name for AcceleratorMode {
    fn name(i: i32) -> Option<&'static str> {
        Some(match i {
            0 => "额外产出",
            1 => "生产加速",
            _ => return None,
        })
    }
}
#[cfg(feature = "visit")]
impl Visit for Lab {
    fn visit<T: super::visit::Visitor + ?Sized>(&mut self, _visitor: &mut T) {}
}
