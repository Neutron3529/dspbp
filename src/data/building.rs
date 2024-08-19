use binrw::{BinRead, BinWrite};
// use num_enum::TryFromPrimitiveError;
#[cfg(feature = "dump")]
use serde::{Deserialize, Serialize};

use super::{belt::Belt, lab::Lab, producer::Producer, sorter::Sorter};
use crate::param::*;

#[cfg(feature = "visit")]
use super::visit::{Visit, Visitor};
use crate::config::{is_v10, round_xy, round_yaw};

// fn b_is(i: ItemId<u16>, f: fn(&DspItem) -> bool) -> bool {
//     i.try_into().as_ref().map(f).unwrap_or_else(|_|{
//         if i.0 as i16 != -1 {println!{"unknown id {}", i.0}}
//         false
//     })
// }

#[cfg_attr(feature = "dump", derive(Serialize, Deserialize))]
#[derive(BinRead, BinWrite, PartialEq, Debug, Clone)]
#[br(import { param_count: usize, building: I16<DspItem> })]
#[br(pre_assert(param_count <= 32768))]
pub enum BuildingParam {
    // #[br(pre_assert(b_is(building, DSPItem::is_station)))]
    // Station(
    //     #[cfg_attr(feature = "verbose", br(dbg))]
    //     #[br(args { is_interstellar: b_is(building, DSPItem::is_interstellar_station), param_count: param_count })]
    //      Station,
    // ),
    #[br(pre_assert(building.is_belt()))]
    Belt(
        #[cfg_attr(feature = "verbose", br(dbg))]
        #[br(if(param_count != 0))]
        #[br(args(param_count))]
        Option<Belt>,
    ),
    #[br(pre_assert(building.is_lab()))]
    Lab(
        #[cfg_attr(feature = "verbose", br(dbg))]
        #[br(if(param_count != 0))]
        #[br(args(param_count))]
        Option<Lab>,
    ),
    #[br(pre_assert(building.is_sorter()))]
    Sorter(
        #[cfg_attr(feature = "verbose", br(dbg))]
        #[br(if(param_count != 0))]
        #[br(args(param_count))]
        Option<Sorter>,
    ),
    #[br(pre_assert(building.is_producer()))]
    Producer(
        #[cfg_attr(feature = "verbose", br(dbg))]
        #[br(if(param_count != 0))]
        #[br(args(param_count))]
        Option<Producer>,
    ),
    Unknown(
        #[cfg_attr(feature = "verbose", br(dbg))]
        #[br(count = param_count)]
        #[br(little)]
        Vec<u32>,
    ),
}

#[cfg_attr(feature = "dump", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "verbose", derive(Debug))]
#[derive(BinRead, BinWrite, Clone)]
#[br(little)]
pub struct BuildingHeader {
    #[brw(if(is_v10()))]
    #[cfg_attr(feature = "dump", serde(default, skip_serializing_if = "is_not_v10"))]
    pub magic_version: i32,
    pub index: i32,
    pub area_index: i8,
    #[br(map=|x:f32|round_xy(x))]
    #[bw(map=|x|round_xy(x))]
    pub local_offset_x: f32,
    #[br(map=|x:f32|round_xy(x))]
    #[bw(map=|x:&f32|round_xy(x))]
    pub local_offset_y: f32,
    #[br(map=|x:f32|round_xy(x))]
    #[bw(map=|x:&f32|round_xy(x))]
    pub local_offset_z: f32,
    #[br(map=|x:f32|round_xy(x))]
    #[bw(map=|x:&f32|round_xy(x))]
    pub local_offset_x2: f32,
    #[br(map=|x:f32|round_xy(x))]
    #[bw(map=|x:&f32|round_xy(x))]
    pub local_offset_y2: f32,
    #[br(map=|x:f32|round_xy(x))]
    #[bw(map=|x:&f32|round_xy(x))]
    pub local_offset_z2: f32,
    #[br(map=|x:f32|round_yaw(x))]
    #[bw(map=|x|round_yaw(x))]
    pub yaw: f32,
    #[br(map=|x:f32|round_yaw(x))]
    #[bw(map=|x|round_yaw(x))]
    pub yaw2: f32,
    #[br(if(is_v10()), map=|x:f32|round_yaw(x))]
    #[bw(if(is_v10()), map=|x|round_yaw(x))]
    #[cfg_attr(feature = "dump", serde(default, skip_serializing_if = "is_not_v10"))]
    pub tilt: f32,
    pub item_id: I16<DspItem>,
    // pub model_index: BPModelId<u16>,
    pub model_index: u16, // TODO: localize it.
    pub output_object_index: i32,
    pub input_object_index: i32,
    pub output_to_slot: i8,
    pub input_from_slot: i8,
    pub output_from_slot: i8,
    pub input_to_slot: i8,
    pub output_offset: i8,
    pub input_offset: i8,
    pub recipe_id: I16<Recipe>,
    pub filter_id: I16<DspItem>,
    pub parameter_count: u16,
}

#[cfg_attr(feature = "dump", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "verbose", derive(Debug))]
#[derive(BinRead, BinWrite, Clone)]
pub struct Building {
    #[cfg_attr(feature = "verbose", br(dbg))]
    pub header: BuildingHeader,
    #[cfg_attr(feature = "verbose", br(dbg))]
    #[br(args { param_count: header.parameter_count as usize, building: header.item_id })]
    pub param: BuildingParam,
    #[brw(ignore)]
    #[serde(skip)]
    pub custom: Edit,
}
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "dump", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "verbose", derive(Debug))]
pub struct Edit {
    pub label: Param<IconId>,
    pub input_idx: i32,  // for beltless mode
    pub output_idx: i32, // for beltless mode
    // pub break_input: bool,  // for beltless mode
    // pub break_output: bool, // for beltless mode
    // pub switch_label: bool, // TODO.
    pub num: f32, // use f32 instead of i32 since DSPGAME use f32 in gameplay (but export i32 data), very large i32 will never be accurate.
}

impl Default for Edit {
    fn default() -> Self {
        Self {
            label: Param(0),
            input_idx: -1,
            output_idx: -1,
            // input_idx: -1,
            // output_idx: -1,
            // break_input: false,
            // break_output: false,
            // switch_label: true,
            num: 0.,
        }
    }
}
#[cfg(feature = "visit")]
impl Visit for Building {
    fn visit<T: Visitor + ?Sized>(&mut self, visitor: &mut T) {
        match &mut self.param {
            // BuildingParam::Station(s) => visitor.visit_station(s),
            BuildingParam::Belt(Some(b)) => visitor.visit_belt(b),
            BuildingParam::Lab(Some(b)) => visitor.visit_lab(b),
            _ => (),
        }
    }
}

// for serde
fn is_not_v10<T>(_: T) -> bool {
    !is_v10()
}
