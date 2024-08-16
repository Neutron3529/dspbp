use clap::{Parser, Subcommand};
/// Dyson Sphere Program blueprint tool.
///
/// For subcommand help, use 'dspbp help <subcommand>'.
#[derive(Parser, Debug)]
#[clap()]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,
    /// Input file. If absent or '-', reads standard input.
    /// When dump/undump mode is used, it is always the blueprint file.
    #[clap(short, long)]
    pub input: Option<String>,
    /// Output file. If absent or '-', writes to standard output.
    /// When dump/undump mode is used, it is always the json file.
    #[clap(short, long)]
    pub output: Option<String>,
    /// Compression level. Uses 9 by default, DSP uses 6. Set it to 9 for about 5% smaller
    /// blueprints that (almost certainly) still work fine.
    #[clap(short, long, default_value_t = 9)]
    pub compression_level: u32,
}

#[derive(Parser, Debug)]
#[clap()]
pub struct EditArgs {
    /// Replace items with other items.
    /// Accepts format like this: "Item1:Replacement1,Item2:Replacement2,..."
    #[clap(short, long)]
    pub replace_item: Option<String>,
    /// Replace recipes with other recipes.
    /// Accepts format like this: "Recipe1:Replacement1,Recipe2:Replacement2,..."
    #[clap(short = 'R', long)]
    pub replace_recipe: Option<String>,
    /// Replace items with other items, also replacing their recipes.
    ///
    /// When there are multiple recipes available, chooses the most basic recipe.
    /// Replacements are overwritten by only-item and only-recipe replacements.
    /// Accepts format like this: "Item1:Replacement1,Item2:Replacement2,..."
    #[clap(short = 'B', long)]
    pub replace_both: Option<String>,
    /// Upgrade/downgrade buildings.
    ///
    /// Accepts format like this: "Building1:Replacement1,Building2:Replacement2,..."
    #[clap(short = 'b', long)]
    pub replace_building: Option<String>,
    /// Replace icon text.
    #[clap(short = 't', long)]
    pub icon_text: Option<String>,
}

#[derive(Parser, Debug)]
#[clap()]
pub struct DumpArgs {
    /// Disable human readable names for IDs of various things.
    /// Ouput without human readable supported CANNOT BE UNDUMPED.
    #[clap(short = 'H', long, default_value_t = true)]
    pub human_readable: bool,
    /// Locale to use. At the moment en and cn are supported. By default, en is used.
    #[clap(short = 'L', long)]
    pub locale: Option<String>,
    /// rounding unit for location xy
    #[clap(short, long, default_value_t = 0.05)]
    pub xy_unit: f64,
    /// rounding unit for angle yaw
    #[clap(short, long, default_value_t = 1.)]
    pub yaw_unit: f64,
    /// do not use rounding, totally ignore the rounding parameters.
    #[clap(short, long, default_value_t = false)]
    pub no_rounding: bool,
    /// verbose mode, output both match and mismatch
    #[clap(short, long, default_value_t = false)]
    pub verbose: bool,
    /// priority unit, the input will divided by its value, the remainder becomes part of priority.
    /// better greater than 0, 0 or less is not tested.
    #[clap(short, long, default_value_t = 1)]
    pub unit: i32,
    /// Belt label that should be regard as broken, SIGNAL-510 (link broken) should be proper.
    #[clap(short, long, default_value_t = -1)]
    pub belt_label: i32,
}

#[derive(Subcommand, Debug, Default)]
pub enum Commands {
    /// Dump blueprint from txt/json to txt/json with specific compression level(if output is txt) and rounding(if specificed).
    #[cfg(feature = "dump")]
    Dump(DumpArgs),
    /// Similar to dump but switch the input and the output.
    #[cfg(feature = "dump")]
    Undump(DumpArgs),
    /// Trigger beltless mode, accept a blueprint in `.txt` or `.json` suffix, output `.json` or `.txt` if the output is not specific
    Beltless(DumpArgs),
    /// Edit blueprint. Accepts more arguments.
    Edit(EditArgs),
    /// Print some blueprint info.
    #[default]
    Info,
    /// Print item names.
    Items,
    /// Print recipe names.
    Recipes,
}
