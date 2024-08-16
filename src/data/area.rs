use binrw::{BinRead, BinWrite};
#[cfg(feature = "dump")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "dump", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "verbose", derive(Debug))]
#[derive(BinRead, BinWrite, Clone)]
#[brw(little)]
pub struct Area {
    index: i8,
    parent_index: i8,
    tropic_anchor: u16,
    area_segments: u16,
    anchor_local_offset_x: u16,
    anchor_local_offset_y: u16,
    width: u16,
    height: u16,
}
