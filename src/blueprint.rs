use std::fmt::Write as _;
use std::io::{Cursor, Read, Write};
use std::str::FromStr;

use crate::data::blueprint::BlueprintData;
#[cfg(feature = "visit")]
use crate::data::visit::{Visit, Visitor};
use crate::error::{some_error, Error};
use crate::config::with_version;
use base64::engine::GeneralPurpose;
use base64::Engine;
use binrw::{BinReaderExt, BinWrite};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
#[cfg(feature = "dump")]
use serde::{Deserialize, Serialize};

use crate::md5::{Algo, MD5Hash, MD5};
use crate::param::*;
#[derive(Clone)]
#[cfg_attr(feature = "dump", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "verbose", derive(Debug))]
pub struct Blueprint {
    pub layout: u32,
    pub icons: [Param<IconId>; 5],
    pub timestamp: u64,
    pub game_version: String,
    pub icon_text: String,
    pub desc: String,
    pub data: BlueprintData,
}

const B64: GeneralPurpose = base64::engine::general_purpose::STANDARD;

impl Blueprint {
    fn int<T: FromStr>(data: &str, what: &str) -> Result<T, Error> {
        str::parse(data).map_err(|_| format!("Failed to parse {}", what).into())
    }

    fn unpack_data(b64data: &str) -> anyhow::Result<(BlueprintData, Vec<u8>)> {
        let zipped_data = B64
            .decode(b64data)
            .map_err(|_| some_error("Failed to base64 decode blueprint"))?;
        let mut d = GzDecoder::new(zipped_data.as_slice());
        let mut data = vec![];
        d.read_to_end(&mut data)?;
        let mut c = Cursor::new(data);
        let out = c.read_le()?;
        Ok((out, c.into_inner()))
    }

    fn hash_str_to_hash(d: &str) -> anyhow::Result<MD5Hash> {
        let d = d.trim();
        if d.len() != 32 {
            return Err(some_error(format!(
                "Unexpected hash length, expected 32, got {}",
                d.len()
            )));
        }
        Ok((0..16)
            .map(|x| (2 * x..2 * x + 2))
            .map(|x| &d[x])
            .map(|x| u8::from_str_radix(x, 16))
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .unwrap())
    }

    fn hash(data: &str) -> MD5Hash {
        MD5::new(Algo::MD5F).process(data.as_bytes())
    }

    fn pack_data(&self, level: Compression) -> anyhow::Result<String> {
        let mut e = GzEncoder::new(Vec::new(), level);
        let mut ws = Cursor::new(vec![]);
        self.data.write_le(&mut ws)?;
        e.write_all(&ws.into_inner()).unwrap();
        let gzipped_data = e.finish().unwrap();
        Ok(B64.encode(gzipped_data.as_slice()))
    }

    pub fn new(data: &str) -> anyhow::Result<Self> {
        let (me, _) = Self::new_with_raw_bp(data)?;
        Ok(me)
    }

    pub fn new_with_raw_bp(data: &str) -> anyhow::Result<(Self, Vec<u8>)> {
        let data_and_hash: Vec<&str> = data.rsplitn(2, "\"").collect();
        if data_and_hash.len() != 2 {
            return Err(some_error("Did not find hash delimiter"));
        }
        let [mut hash, mut data]: [&str; 2] = data_and_hash.try_into().unwrap();
        hash = hash.trim();
        data = data.trim();

        // NOTICE: we hash the blueprint without the trailing quote!
        let hash = Self::hash_str_to_hash(hash)?;
        let our_hash = Self::hash(data);
        if hash != our_hash {
            return Err(some_error(format!(
                "Blueprint hash does not match calculated hash: {:x?} != {:x?}",
                hash, our_hash
            )));
        }

        const PREFIX: &str = "BLUEPRINT:";
        if data.len() < PREFIX.len() || &data[0..PREFIX.len()] != PREFIX {
            let ml = std::cmp::min(PREFIX.len(), data.len());
            return Err(some_error(format!("Unexpected prefix: {}", &data[0..ml])));
        }
        data = &data[PREFIX.len()..];

        let fields: Vec<&str> = data.split(',').collect();
        if fields.len() != 12 {
            return Err(some_error(format!(
                "Expected 12 CSV elements, got {}",
                fields.len()
            )));
        }

        let [fixed0_1, layout]: [&str; 2] = fields[0..2].try_into().unwrap();
        let icons = &fields[2..7];
        let [fixed0_2, timestamp, game_version, icon_text, desc_plus_b64data]: [&str; 5] =
            fields[7..12].try_into().unwrap();
        let [desc, b64data]: [&str; 2] = desc_plus_b64data
            .split('"')
            .collect::<Vec<&str>>()
            .try_into()
            .unwrap();

        let icon_text = urlencoding::decode(icon_text)
            .map(|x| x.into_owned())
            .unwrap_or_else(|_| desc.to_string());
        let desc = urlencoding::decode(desc)
            .map(|x| x.into_owned())
            .unwrap_or_else(|_| desc.to_string());
        let fixed0_1: u32 = Self::int(fixed0_1, "fixed0_1")?;
        let layout = Self::int(layout, "layout")?;
        let icons: Vec<Param<IconId>> = icons
            .into_iter()
            .map(|&x| Param(IconId::de(x)))
            .collect::<Vec<Param<IconId>>>();
        let fixed0_2: u32 = Self::int(fixed0_2, "fixed0_2")?;
        let timestamp = Self::int(timestamp, "timestamp")?;

        if fixed0_1 != 0 {
            return Err(some_error("fixed0_1 is not 0"));
        }
        if fixed0_2 != 0 {
            return Err(some_error("fixed0_2 is not 0"));
        }

        let (data, raw_bp) = with_version(game_version, || Self::unpack_data(b64data))?;

        Ok((
            Self {
                layout,
                icons: icons.try_into().unwrap(),
                timestamp,
                game_version: game_version.into(),
                icon_text: icon_text.into(),
                desc: desc.into(),
                data,
            },
            raw_bp,
        ))
    }
    pub fn into_bp_string(&self, level: u32) -> anyhow::Result<String> {
        let icons = self.icons.map(|x| x.0.to_string()).join(",");
        let mut out = with_version(&self.game_version, || {
            format!(
                "BLUEPRINT:0,{},{},0,{},{},{},{}\"{}",
                self.layout,
                icons,
                self.timestamp,
                self.game_version,
                urlencoding::encode(&self.icon_text),
                urlencoding::encode(&self.desc),
                self.pack_data(Compression::new(level))
                    .expect("cannot compress the data")
            )
        });
        let hash = Self::hash(&out);
        write!(&mut out, "\"").unwrap();
        for b in hash {
            write!(&mut out, "{:02X}", b).unwrap();
        }
        Ok(out)
    }
    // pub fn txt_version(txt: &str) -> &str {
    //     txt.split(r#"""#).nth(4).unwrap()
    // }

    #[cfg(feature = "dump")]
    pub fn json_version(json: &str) -> &str {
        json.split_once(r#"game_version"#)
            .expect("cannot find game version")
            .1
            .split(r#"""#)
            .nth(1)
            .expect("game version does not contains '\"'")
    }

    #[cfg(feature = "dump")]
    pub fn new_from_json(json: &str) -> anyhow::Result<Self> {
        with_version(Self::json_version(json), || Ok(serde_json::from_str(json)?))
    }

    #[cfg(feature = "dump")]
    pub fn dump_json(&self) -> anyhow::Result<Vec<u8>> {
        with_version(&self.game_version, || Ok(serde_json::to_vec(self)?))
    }

    #[cfg(feature = "dump")]
    pub fn dump_json_pretty(&self) -> anyhow::Result<Vec<u8>> {
        with_version(&self.game_version, || Ok(serde_json::to_vec_pretty(self)?))
    }

    // pub fn get_description(&self) -> anyhow::Result<String> {
    //     Ok(self.desc.to_owned())
    // }

    // pub fn set_icon_text(&mut self, text: &str) {
    //     self.icon_text = text.to_owned();
    // }

    // pub fn get_icon_text(&self) -> anyhow::Result<String> {
    //     Ok(self.icon_text.to_owned())
    // }
}
#[cfg(feature = "visit")]
impl Visit for Blueprint {
    fn visit<T: Visitor + ?Sized>(&mut self, visitor: &mut T) {
        visitor.visit_blueprint_data(&mut self.data)
    }
}
