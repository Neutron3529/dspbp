use args::{Args, Commands, DumpArgs};
use blueprint::Blueprint;
use clap::Parser;
use data::traits::TryFromUserString;
// use edit::EditBlueprint;
use locale::{Locale, GLOBAL_SERIALIZATION_LOCALE};
use std::{
    fs::File,
    io::{Cursor, Read, Seek, Stdout, Write},
};
// use crate::{data::visit::Visitor};
use crate::data::building::{Building, BuildingParam::*};
use crate::thread_local::with_rounding;
use std::array;
use std::collections::{BinaryHeap, HashMap};
// use crate::param::*;
pub(crate) mod args;
pub(crate) mod blueprint;
pub(crate) mod data;
// pub(crate) mod edit;
pub(crate) mod error;
pub(crate) mod locale;
pub(crate) mod md5;
// #[cfg(feature = "python")]
// pub(crate) mod python;
// pub(crate) mod stats;
#[cfg(test)]
pub(crate) mod testutil;

fn iof(arg: &Option<String>) -> Option<&str> {
    match arg.as_ref().map(|x| x.as_ref()) {
        None | Some("-") => None,
        file => file,
    }
}

pub trait ReadPlusSeek: Read + Seek {}
impl<T: Read + Seek> ReadPlusSeek for T {}
pub trait WritePlusSeek: Write + Seek {}
impl<T: Write + Seek> WritePlusSeek for T {}

pub enum WriteSeek {
    File(File),
    BufOut(Cursor<Vec<u8>>, Stdout),
}

impl WriteSeek {
    fn flush_if_stdout(&mut self) -> std::io::Result<()> {
        if let Self::BufOut(c, s) = self {
            s.write_all(c.get_ref().as_ref())
        } else {
            Ok(())
        }
    }
}

impl Write for WriteSeek {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::File(f) => f.write(buf),
            Self::BufOut(c, _) => c.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::File(f) => f.flush(),
            Self::BufOut(c, _) => c.flush(),
        }
    }
}

impl Seek for WriteSeek {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match self {
            Self::File(f) => f.seek(pos),
            Self::BufOut(c, _) => c.seek(pos),
        }
    }
}

fn itob(i: &mut Box<dyn ReadPlusSeek>) -> anyhow::Result<Blueprint> {
    let mut data = vec![];
    i.read_to_end(&mut data)?;
    let data = String::from_utf8(data)?;
    Blueprint::new(&data)
}

/// Remover是必要的，因为戴森球会将index不连续的蓝图识别为无效蓝图
pub struct Remover<'a>(
    /// data
    &'a mut Vec<Building>,
    /// rev index, data[this[index]].index == index
    HashMap<i32, i32>,
    /// belt indexes
    Vec<i32>,
    /// sorter indexes
    Vec<i32>,
    /// Vec count modifier.
    &'a mut u32,
);
impl<'a> Remover<'a> {
    pub fn new(v: &'a mut crate::data::blueprint::BlueprintData) -> Self {
        let mut ret = Self(
            &mut v.buildings,
            HashMap::new(),
            Vec::new(),
            Vec::new(),
            &mut v.building_count,
        );
        for (n, i) in ret.0.iter().enumerate() {
            assert_eq! {n as i32,i.header.index}
            if i.header.item_id.is_belt() {
                ret.2.push(n as i32)
            } else if i.header.item_id.is_sorter() {
                ret.3.push(n as i32)
            }
        }
        ret
    }
    pub fn remove(&mut self, i: i32) {
        let i = self.1.get(&i).copied().unwrap_or(i);
        // suppose we have [1, 3, 5], self.0[self.1[1]] = 1 => self.1 = {1:0,3:1,5:2}
        let removed_idx = self.0[i as usize].header.index;
        // suppose 1:0 is going to be removed, thus i=0, we will have [5, 3]
        // thus idx = self.0[0]..index = 1

        self.0.swap_remove(i as usize);
        *self.1.entry(removed_idx).or_default() = -1;
        // should be {3:1, 5:0}

        if self.0.len() > i as usize {
            *self.1.entry(self.0[i as usize].header.index).or_default() = i;
        }
    }
    pub fn drop(self) {
        *self.4 = self.0.len() as u32;
        self.0.iter_mut().for_each(|b| {
            for x in [
                &mut b.header.index,
                &mut b.header.output_object_index,
                &mut b.header.input_object_index,
            ] {
                *x = *self.1.get(x).unwrap_or(x)
            }
        });
        assert!(self.0.is_sorted_by_key(|x| x.header.index));
    }
}
impl<'a> std::ops::Index<i32> for Remover<'a> {
    type Output = Building;
    fn index(&self, index: i32) -> &Self::Output {
        &self.0[self.1.get(&index).copied().unwrap_or(index) as usize]
    }
}
impl<'a> std::ops::IndexMut<i32> for Remover<'a> {
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        &mut self.0[self.1.get(&index).copied().unwrap_or(index) as usize]
    }
}

pub fn input(args: &Args) -> anyhow::Result<Box<dyn ReadPlusSeek>> {
    match iof(&args.input) {
        None => {
            let mut all_input = vec![];
            eprintln!("Reading blueprint from standard input.");
            std::io::stdin().read_to_end(&mut all_input)?;
            Ok(Box::new(Cursor::new(all_input)))
        }
        Some(file) => Ok(Box::new(std::fs::File::open(file)?)),
    }
}
pub fn output(args: &Args, suffix: &str) -> anyhow::Result<WriteSeek> {
    match iof(&args.output) {
        None => Ok(WriteSeek::BufOut(Cursor::new(vec![]), std::io::stdout())),
        Some(file) => {
            let ext = file.rfind(".").unwrap_or(file.len());
            let mut file = file.to_string();
            file.insert_str(ext, suffix);
            Ok(WriteSeek::File(
                std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(file)?,
            ))
        }
    }
}

#[cfg(feature = "dump")]
pub fn dump(args: &Args, dump: &DumpArgs) -> anyhow::Result<()> {
    let _ = match &dump.locale {
        None => GLOBAL_SERIALIZATION_LOCALE.set(Locale::cn),
        Some(s) => GLOBAL_SERIALIZATION_LOCALE.set(Locale::try_from_user_string(s)?),
    };
    with_rounding(
        if dump.no_rounding {
            [0.; 2]
        } else {
            [dump.xy_unit, dump.yaw_unit]
        },
        || {
            let mut input = input(&args)?;
            let mut output = output(&args, "")?;
            let bp = itob(&mut input)?;

            if dump.human_readable {
                output.write_all(&bp.dump_json_pretty()?)?;
            } else {
                output.write_all(&bp.dump_json()?)?;
            }
            Ok::<(), anyhow::Error>(output.flush_if_stdout()?)
        },
    )
}
#[cfg(feature = "dump")]
pub fn undump(args: &Args, dump: &DumpArgs) -> anyhow::Result<()> {
    let _ = match &dump.locale {
        None => GLOBAL_SERIALIZATION_LOCALE.set(Locale::cn),
        Some(s) => GLOBAL_SERIALIZATION_LOCALE.set(Locale::try_from_user_string(s)?),
    };
    with_rounding(
        if dump.no_rounding {
            [0.; 2]
        } else {
            [dump.xy_unit, dump.yaw_unit]
        },
        || {
            let mut data = vec![];
            let mut input = input(&args)?;
            let mut output = output(&args, "")?;
            input.read_to_end(&mut data)?;
            let data = String::from_utf8(data)?;
            let bp = Blueprint::new_from_json(&data)?;
            output.write_all(bp.into_bp_string(args.compression_level)?.as_bytes())?;
            Ok::<(), anyhow::Error>(output.flush_if_stdout()?)
        },
    )
}

pub fn beltless(args: &Args, dump: &DumpArgs) -> anyhow::Result<()> {
    let output_is_json = args
        .output
        .as_ref()
        .map(|x| x.ends_with(".json"))
        .unwrap_or(false);
    let input_is_json = args
        .input
        .as_ref()
        .map(|x| x.ends_with(".json"))
        .unwrap_or(false);
    with_rounding(
        if dump.no_rounding {
            [0.; 2]
        } else {
            [dump.xy_unit, dump.yaw_unit]
        },
        || {
            let _ = match &dump.locale {
                None => GLOBAL_SERIALIZATION_LOCALE.set(Locale::cn),
                Some(s) => GLOBAL_SERIALIZATION_LOCALE.set(Locale::try_from_user_string(s)?),
            };
            let mut input = input(&args)?;

            let mut bp = if input_is_json {
                let mut data = vec![];
                input.read_to_end(&mut data)?;
                let data = String::from_utf8(data)?;
                Blueprint::new_from_json(&data)?
            } else {
                itob(&mut input)?
            };
            // preprocess, move belt label to sorter

            let mut cntr = [0; 3];
            let mut scnt = [0; 3];
            let mut sorter_labels = HashMap::new();
            let mut belt_labels = HashMap::new();
            let [mut sm, mut bm] = array::from_fn(|_| HashMap::new());
            let mut rm = Remover::new(&mut bp.data);
            #[derive(Clone, Copy, PartialEq, Eq)]
            struct InOut(
                i32, // id
                i32, // count
                i32, // chance
            );
            impl std::fmt::Debug for InOut {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}({}x{})", self.1, self.0, self.2)
                }
            }

            impl PartialOrd for InOut {
                fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
                    Some(self.cmp(&o))
                }
            }
            impl Ord for InOut {
                fn cmp(&self, o: &Self) -> std::cmp::Ordering {
                    self.1
                        .cmp(&o.1) // count多的优先
                        .then(o.2.cmp(&self.2)) // chance少的优先，因为是最大匹配，还剩1次的匹配成功的概率最高
                        .then(o.0.cmp(&self.0)) // id小的优先
                }
            }
            for cnt in rm.2.iter() {
                // belts
                let b = &mut rm.0[*rm.1.get(cnt).unwrap_or(cnt) as usize];
                let idx = b.header.index;
                let input = b.header.input_object_index;
                let output = b.header.output_object_index;
                if b.header.item_id.is_belt() {
                    cntr[2] += 1;
                } else {
                    panic!(
                        "Remover goes wrong, cnt={cnt} -> rm.1[cnt]={} -> b is not belt.",
                        rm.1[cnt]
                    )
                }
                if let Belt(Some(ref belt)) = b.param {
                    cntr[1] += 1;
                    b.custom.label = belt.label;
                    b.custom.num = belt.count as f32;

                    belt_labels
                        .entry(belt.label)
                        .or_insert(HashMap::new())
                        .insert(idx, belt.count as i32);
                    if input == -1 && output == -1 {
                        cntr[0] += 1;
                    }
                } else if let Belt(None) = b.param {
                    // normal
                } else {
                    panic! {"传送带里面混进来一些奇奇怪怪的东西: {:?}", b.param}
                }
            }
            println! {"belt: {} pseudo orphan, {} labelled, {} total. Labels={belt_labels:?}",cntr[0], cntr[1], cntr[2]}
            for cnt in rm.3.iter() {
                // sorters
                let b = &mut rm.0[*rm.1.get(cnt).unwrap_or(cnt) as usize];
                if !b.header.item_id.is_sorter() {
                    panic!(
                        "Remover goes wrong, cnt={cnt} -> rm.1[cnt]={} -> b is not sorter.",
                        rm.1[cnt]
                    )
                }
                if b.header.filter_id.0 == 0 {
                    continue; // ignore sorter without filter.
                }
                scnt[2] += 1;
                let filter = b.header.filter_id;
                let idx = b.header.index;
                let input = b.header.input_object_index;
                let output = b.header.output_object_index;
                let belt_input = input >= 0 && rm[input].custom.label.0 != 0;
                let belt_output = output >= 0 && rm[output].custom.label.0 != 0;
                if belt_input ^ belt_output {
                    let belt =
                        rm.0.get(if belt_input {
                            scnt[0] += 1;
                            input
                        } else {
                            scnt[1] += 1;
                            output
                        } as usize)
                            .unwrap();
                    let (bidx, belt) = (belt.header.index, &belt.param);
                    if let Belt(Some(belt)) = belt {
                        // 这些分拣器对应的传送带是用作标记的节点，理应删除
                        if let Some(Some(idx)) =
                            belt_labels.get_mut(&belt.label).map(|x| x.remove(&bidx))
                        {
                            if dump.verbose {
                                println!(
                                    "已将{bidx}节点识别为数量{idx}的{}{}端",
                                    belt.label,
                                    if belt_input { "输入" } else { "输出" }
                                )
                            }
                        } else {
                            eprintln!("未清空分拣器({idx})输入端连接传送带({bidx})的标记")
                        }
                        let cur = *if belt_input { &mut bm } else { &mut sm }
                            .entry(belt.label)
                            .or_insert(filter);
                        sorter_labels
                            .entry(belt.label)
                            .or_insert(BinaryHeap::new())
                            .push(InOut(idx, belt.count as i32, 1));
                        if cur != filter {
                            panic! {"label reuse founded, label {} accepts both {cur} and {}",belt.label,filter}
                        }
                    }
                } else {
                    eprintln!("分拣器{}的输入与输出均接触到带标记的传送带？", idx)
                }
            }
            println! {"sorter: {} consumer, {} producer, {} total.",scnt[0], scnt[1], scnt[2]}
            println! {"sorter output map:{sm:?}\nbelt output map:{bm:?}"}
            println! {"sorter count:{sorter_labels:?}"};
            if sm.keys().any(|x| bm.contains_key(x)) {
                panic!("输出集合{sm:?}与输入集合{bm:?}含有同样标记{:?}，作者暂时没有处理这个情形的打算", sm.keys().filter(|&x|bm.contains_key(x)).collect::<Vec<_>>())
            }
            let mut belt_labels: HashMap<_, BinaryHeap<InOut>> = belt_labels
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().map(|(k, v)| InOut(k, v, 8)).collect()))
                .collect();

            println!("传送带标记：{belt_labels:?}");
            for (state, label) in [(true, sm), (false, bm)]
                .into_iter()
                .flat_map(|x| x.1.into_iter().map(move |y| (x.0, y.0)))
            {
                let Some(mut iter1) = sorter_labels.remove(&label) else {
                    eprintln!("在sorter_labels中找不到{label}，sorter_labels={sorter_labels:?}");
                    continue;
                };
                let Some(mut iter2) = belt_labels.remove(&label) else {
                    eprintln!("在belt_labels中找不到{label}, belt_labels={belt_labels:?}");
                    continue;
                };
                while let Some(oi) = iter1.pop() {
                    if let Some(mut io) = iter2.pop() {
                        if state {
                            rm.remove(rm[oi.0].header.output_object_index);
                            rm[oi.0].header.output_object_index = io.0;
                        } else {
                            rm.remove(rm[oi.0].header.input_object_index);
                            rm[oi.0].header.input_object_index = io.0;
                        }
                        if dump.verbose {
                            println! {"匹配: 分拣器{}的{label} {} 传送带{}(count {} -> {}, {} remains)", oi.0, if state { ":->" } else { "<-:" }, io.0, io.1, io.1-oi.1, io.2-1}
                        }
                        io.1 -= (oi.1 / dump.unit) * dump.unit;
                        io.2 -= 1;
                        if io.2 > 0 && (io.1) / dump.unit > 0 {
                            iter2.push(io)
                        }
                    } else {
                        iter1.push(oi);
                        break;
                    }
                }
                if iter1.len() > 0 {
                    println!(
                        "未匹配{label}的分拣器{} {:?}",
                        if state { "输出" } else { "输入" },
                        iter1.into_iter().collect::<Vec<_>>()
                    );
                }
                if iter2.len() > 0 {
                    println!(
                        "未匹配{label}的传送带{} {:?}",
                        if state { "输入" } else { "输出" },
                        iter2.into_iter().collect::<Vec<_>>()
                    );
                }
            }
            rm.drop();

            loop {
                if dump.belt_label == -1 {
                    break;
                }
                // output B
                let mut bp = bp.clone();
                bp.desc += "-B";

                // 删除传送带相关物品
                // // 标记传送带
                let mut rm = Remover::new(&mut bp.data);
                let mut idx = -1;
                for i in rm.2.iter().copied() {
                    if rm[i].custom.label.0 == dump.belt_label {
                        idx = i;
                        break;
                    }
                }
                if idx == -1 {
                    println!("Warning: 找不到label为{}的传送带", dump.belt_label);
                    break rm.drop();
                }
                // // 删除标记
                let mut len = rm.0.len();
                while len > 0 {
                    len -= 1;
                    if rm.0[len].header.item_id.is_belt_related() {
                        if len as i32 != idx {
                            rm.remove(len as i32);
                        }
                    }
                }
                rm.drop();

                // 开始重定向失效爪子
                // // 重新标记传送带
                let rm = Remover::new(&mut bp.data);
                let mut idx = -1;
                for i in rm.2.iter().copied() {
                    if rm[i].custom.label.0 == dump.belt_label {
                        idx = i;
                        break;
                    }
                }
                if idx == -1 {
                    unreachable!("在阶段2找不到label为{}的传送带", dump.belt_label);
                }

                let mut len = rm.3.len();
                while len > 0 {
                    len -= 1;
                    if rm.0[rm.3[len] as usize].header.input_object_index == -1 {
                        rm.0[rm.3[len] as usize].header.input_object_index = idx;
                    }
                    if rm.0[rm.3[len] as usize].header.output_object_index == -1 {
                        rm.0[rm.3[len] as usize].header.output_object_index = idx;
                    }
                }
                rm.drop();

                let mut output = output(&args, "-A")?;
                if output_is_json {
                    output.write_all(&bp.dump_json_pretty()?)?;
                } else {
                    output.write_all(bp.into_bp_string(args.compression_level)?.as_bytes())?;
                }
                break;
            }

            let mut output = output(&args, if dump.belt_label == -1 { "" } else { "-B" })?;
            if output_is_json {
                output.write_all(&bp.dump_json_pretty()?)?;
            } else {
                output.write_all(bp.into_bp_string(args.compression_level)?.as_bytes())?;
            }
            Ok::<(), anyhow::Error>(output.flush_if_stdout()?)
        },
    )
}

pub fn cmdline() -> anyhow::Result<()> {
    let mut args = args::Args::parse();
    if let &Commands::Undump(_) = &args.command {
        (args.input, args.output) = (args.output, args.input)
    }
    if args.output.is_none() {
        if let Some(input) = args
            .input
            .as_ref()
            .map(|x| x.strip_suffix(".txt"))
            .flatten()
        {
            args.output = Some(format!("{input}.json"))
        }
    }

    match args.command {
        #[cfg(feature = "dump")]
        Commands::Dump(ref dump) => crate::dump(&args, dump)?,
        #[cfg(feature = "dump")]
        Commands::Undump(ref dump) => crate::undump(&args, dump)?,
        Commands::Beltless(ref dump) => crate::beltless(&args, dump)?,
        // Commands::Edit(eargs) => {
        // }
        // Commands::Info => {
        // }
        // Commands::Items => {
        //     for e in DSPItem::iter() {
        //         println!("{}", e.as_ref())
        //     }
        // }
        // Commands::Recipes => {
        //     for e in DSPRecipe::iter() {
        //         println!("{}", e.as_ref())
        //     }
        // }
        _ => eprintln!("Unsupported now"),
    }
    Ok(())
}

pub mod thread_local {
    use std::cell::Cell;
    thread_local! {
        static UNIT: Cell<[f64;2]> = Cell::new([0.;2]);
    }
    pub fn with_rounding<T, F: FnOnce() -> T>(unit: [f64; 2], f: F) -> T {
        struct Scoped([f64; 2]);
        impl Drop for Scoped {
            fn drop(&mut self) {
                UNIT.with(|cx| cx.set(self.0));
            }
        }
        let _old = UNIT.with(|cx| {
            let prev = cx.get();
            cx.set(unit);
            Scoped(prev)
        });
        f()
    }
    /// Get the current locale. Panics if there isn't one.
    pub fn get_unit(i: usize) -> f64 {
        UNIT.with(|cx| cx.get())[i]
    }
    /// Get the current locale. Panics if there isn't one.
    pub fn round(val: f32, ty: usize) -> f32 {
        let unit = get_unit(ty);
        if unit <= 0. {
            val
        } else {
            ((val as f64 / unit).round_ties_even() * unit + 0.) as f32 // avoid -0.0
        }
    }
    pub fn round_xy(val: impl std::borrow::Borrow<f32>) -> f32 {
        round(*val.borrow(), 0)
    }
    pub fn round_yaw(val: impl std::borrow::Borrow<f32>) -> f32 {
        round(*val.borrow(), 1)
    }

    thread_local! {
        static VERSION: Cell<i32> = Cell::new(0);
    }
    pub fn with_version<T, F: FnOnce() -> T>(version: &str, f: F) -> T {
        struct Scoped(i32);
        impl Drop for Scoped {
            fn drop(&mut self) {
                VERSION.with(|cx| cx.set(self.0));
            }
        }

        let detailed_version = version
            .split('.')
            .map(|x| x.parse::<i32>().unwrap_or(0))
            .collect::<Vec<i32>>();
        let mut version = 0;
        if detailed_version >= vec![0, 10, 30, 22239] {
            version = V10
        }

        let _old = VERSION.with(|cx| {
            let prev = cx.get();
            cx.set(version);
            Scoped(prev)
        });
        f()
    }
    pub static V10: i32 = 1;
    pub fn is_v10() -> bool {
        VERSION.with(|cx| cx.get()) >= V10
    }
}
pub mod param;
