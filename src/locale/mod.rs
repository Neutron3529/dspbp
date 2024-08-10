use std::{
    collections::HashMap,
    sync::{LazyLock, OnceLock},
};

use crate::data::traits::TryFromUserString;

// static GLOBAL_DATA: OnceLock<Mutex<HashMap<i32, &'static str>>> = OnceLock::new();
// pub fn unknown(t:i32)->&'static str {
//     &*GLOBAL_DATA.get_or_init(||Mutex::new(HashMap::new())).lock().unwrap().entry(t).or_insert(format!("Unknown({})",t).leak())
// }

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum Locale {
    cn,
    en,
}

impl TryFromUserString for Locale {
    fn try_from_user_string(s: &str) -> anyhow::Result<Self> {
        match s {
            "cn" => Ok(Locale::cn),
            "en" => Ok(Locale::en),
            x => anyhow::bail!("Unknown locale {x}. Supported locales: cn, en."),
        }
    }
}

pub(crate) static GLOBAL_SERIALIZATION_LOCALE: OnceLock<Locale> = OnceLock::new();

// struct LList<T: 'static>(Locale, &'static [(T, &'static str)]);
type LList<T> = (Locale, &'static [(T, &'static str)]);

static DSP_ITEM_LLIST: &[LList<i32>] = &[
    (Locale::en, include!("data/en/items.rs")),
    (Locale::cn, include!("data/cn/items.rs")),
];

static DSP_RECIPE_LLIST: &[LList<i32>] = &[
    (Locale::en, include!("data/en/recipes.rs")),
    (Locale::cn, include!("data/cn/recipes.rs")),
];

macro_rules! localized_enum_impl {
    ($table: ident, $source: ident) => {
        pub(crate) static $table: LazyLock<HashMap<Locale, HashMap<i32, &'static str>>> =
            LazyLock::new(|| {
                HashMap::from_iter(
                    $source
                        .iter()
                        .copied()
                        .map(|(k, v)| (k, HashMap::from_iter(v.into_iter().copied()))),
                )
            });
    };
}

localized_enum_impl!(DSP_ENUM_LOCALE, DSP_ITEM_LLIST);
localized_enum_impl!(DSP_RECIPE_LOCALE, DSP_RECIPE_LLIST);
// FIXME how to extract model names from DSP? Console has no commands for that.
// static DSP_MODEL_LLIST: &[LList<i32>] = &[];
// localized_enum_impl!(BP_MODEL_LOCALE, DSP_MODEL_LLIST);
