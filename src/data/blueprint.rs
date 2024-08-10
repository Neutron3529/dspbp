use binrw::{BinRead, BinWrite};
#[cfg(feature = "dump")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "visit")]
use super::visit::{Visit, Visitor};
use crate::data::{area::Area, building::Building};

#[cfg_attr(feature = "dump", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "verbose", derive(Debug))]
#[derive(BinRead, BinWrite)]
#[br(little)]
pub struct Header {
    version: u32,
    cursor_offset_x: u32,
    cursor_offset_y: u32,
    cursor_target_area: u32,
    dragbox_size_x: u32,
    dragbox_size_y: u32,
    primary_area_index: u32,
    area_count: u8,
}

#[cfg_attr(feature = "dump", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "verbose", derive(Debug))]
#[derive(BinRead, BinWrite)]
#[br(little)]
pub struct BlueprintData {
    #[br(assert(header.version == 1))]
    pub header: Header,
    #[br(count = header.area_count)]
    pub areas: Vec<Area>,
    pub building_count: u32,
    #[br(count = building_count)]
    pub buildings: Vec<Building>,
}
#[cfg(feature = "visit")]
impl Visit for BlueprintData {
    fn visit<T: Visitor + ?Sized>(&mut self, visitor: &mut T) {
        for b in self.buildings.iter_mut() {
            visitor.visit_building(b)
        }
    }
}
