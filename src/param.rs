use crate::data::traits::Nice;
use binrw::{BinRead, BinWrite};
#[cfg(feature = "dump")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;
use std::{fmt, marker::PhantomData};

pub trait Name: Sized {
    fn name(i: i32) -> Option<&'static str>;
    fn get_name(i: i32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(s) = <Self as Name>::name(i) {
            write!(f, "{s}({i})")
        } else {
            write!(f, "Unknown({i})")
        }
    }
    fn ser<'a, G: Into<i32> + Nice + ToString + FromStr>(
        s: &'a GenericParam<Self, G>,
    ) -> &'a impl std::fmt::Display
    where
        GenericParam<Self, G>: Sized,
    {
        s
    }
    fn de<G: FromStr>(s: &str) -> G {
        let first = s.rsplit_once('(').unwrap_or(("", "")).1;
        let second = first.rsplit_once(')').unwrap_or(("", "")).0;
        let Ok(res) = second.parse::<G>() else {
            if let Ok(res) = s.parse::<G>() {
                return res;
            }
            panic!("cannot deserilize {s}, first = `{first}`, second = `{second}` ",)
        };
        res
    }
}

#[derive(BinRead, BinWrite, Clone, Copy)]
#[brw(little)]
pub struct GenericParam<T: Name, G: Into<i32> + Nice + ToString + FromStr>(
    pub G,
    #[brw(ignore)] pub [PhantomData<T>; 0],
);

impl<T: Name, G: Into<i32> + ToString + Nice + FromStr> std::fmt::Display for GenericParam<T, G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <T as Name>::get_name(self.0.into(), f)
    }
}
impl<T: Name, G: Into<i32> + ToString + Nice + FromStr> std::fmt::Debug for GenericParam<T, G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <T as Name>::get_name(self.0.into(), f)
    }
}

// manually derive
impl<T: Name, G: Into<i32> + ToString + Nice + FromStr> Eq for GenericParam<T, G> {}
impl<T: Name, G: Into<i32> + ToString + Nice + FromStr> PartialEq for GenericParam<T, G> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<T: Name, G: Into<i32> + ToString + Nice + FromStr> Ord for GenericParam<T, G> {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        self.0.cmp(&o.0)
    }
}
impl<T: Name, G: Into<i32> + ToString + Nice + FromStr> PartialOrd for GenericParam<T, G> {
    fn partial_cmp(&self, o: &Self) -> std::option::Option<std::cmp::Ordering> {
        self.0.partial_cmp(&o.0)
    }
}
impl<T: Name, G: Into<i32> + ToString + Nice + FromStr + std::hash::Hash> std::hash::Hash
    for GenericParam<T, G>
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

pub type Param<T> = GenericParam<T, i32>;
pub type I16<T> = GenericParam<T, i16>;
#[allow(non_snake_case)]
pub fn Param<T: Name>(i: i32) -> Param<T> {
    GenericParam(i, [])
}
#[allow(non_snake_case)]
pub fn I16<T: Name>(i: i16) -> I16<T> {
    GenericParam(i, [])
}
#[cfg(feature = "dump")]
impl<T: Name, G: Into<i32> + ToString + Nice + FromStr> Serialize for GenericParam<T, G> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(T::ser(self))
    }
}

#[cfg(feature = "dump")]
impl<'de, T: Name, G: Into<i32> + ToString + Nice + FromStr> Deserialize<'de>
    for GenericParam<T, G>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Ok(a) = String::deserialize(deserializer) else {
            panic!("cannot deserilize string");
        };
        Ok(GenericParam(T::de(&a), []))
    }
}

#[derive(Clone, Copy)]
pub struct DspItem;
impl crate::param::Name for DspItem {
    fn name(i: i32) -> Option<&'static str> {
        crate::locale::DSP_ENUM_LOCALE
            .get(
                crate::GLOBAL_SERIALIZATION_LOCALE
                    .get()
                    .unwrap_or(&crate::Locale::cn),
            )?
            .get(&i)
            .map(|v| *v)
    }
}
#[allow(non_upper_case_globals)]
impl I16<DspItem> {
    pub const ConveyorBeltMKI: i16 = 2001;
    pub const ConveyorBeltMKIII: i16 = 2003;
    pub const MatrixLab: i16 = 2901;
    pub const SelfevolutionLab: i16 = 2902;
    pub const SorterMKI: i16 = 2011;
    pub const SorterMKIV: i16 = 2014;
    pub fn is_belt(&self) -> bool {
        Self::ConveyorBeltMKI <= self.0 && self.0 <= Self::ConveyorBeltMKIII
    }
    pub fn is_lab(&self) -> bool {
        Self::MatrixLab <= self.0 && self.0 <= Self::SelfevolutionLab
    }
    pub fn is_sorter(&self) -> bool {
        Self::SorterMKI <= self.0 && self.0 <= Self::SorterMKIV
    }
}
pub struct Recipe;
impl Name for Recipe {
    fn name(i: i32) -> Option<&'static str> {
        crate::locale::DSP_RECIPE_LOCALE
            .get(
                crate::GLOBAL_SERIALIZATION_LOCALE
                    .get()
                    .unwrap_or(&crate::Locale::cn),
            )?
            .get(&i)
            .map(|v| *v)
    }
}
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum IconId {
    Signal(i32),
    Item(i16),
    Recipe(i16),
    Tech(i32),
    Unknown(i32),
}
impl TryFrom<i32> for IconId {
    type Error = anyhow::Error;
    fn try_from(n: i32) -> Result<Self, Self::Error> {
        let me = if n < 1000 {
            Self::Signal(n)
        } else if n < 20000 {
            Self::Item(n as i16)
        } else if n < 40000 {
            Self::Recipe((n - 20000) as i16)
        } else if n < 60000 {
            Self::Tech(n - 40000)
        } else {
            Self::Unknown(n)
        };
        Ok(me)
    }
}

impl From<IconId> for i32 {
    fn from(value: IconId) -> Self {
        match value {
            IconId::Signal(v) => v,
            IconId::Item(v) => v.into(),
            IconId::Recipe(v) => v as i32 + 20000,
            IconId::Tech(v) => v + 40000,
            IconId::Unknown(v) => v,
        }
    }
}
impl Name for IconId {
    fn name(i: i32) -> Option<&'static str> {
        match i.try_into() {
            Ok(IconId::Item(i)) => DspItem::name(i as i32),
            Ok(IconId::Recipe(i)) => Recipe::name(i as i32),
            Ok(IconId::Signal(i)) => Some(if i == 0 { "无" } else { "标记" }),
            Ok(IconId::Tech(_)) => Some("科技"),
            _ => None,
        }
    }
}
