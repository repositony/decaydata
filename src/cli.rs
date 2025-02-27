// internal
use crate::wrappers::{CliRadType, Property};

// command line modules
use clap::builder::styling::{AnsiColor, Effects};
use clap::builder::Styles;
use clap::{arg, Parser};

// other
use anyhow::Result;

/// Retrieve decay data from the IAEA chart of nuclides
///
/// Examples
/// --------
///
///  Typical use:
///     $ ddata co60 cs137 ag108m
///
///  Nuclide formats:
///     $ ddata co60 Co60 CO60 Co60m0 => Ground state Co60
///     $ ddata co60m co60m1 co60*    => First excited state Co60
///     $ ddata co                    => All Co ground state isotopes
///
///  Writing data to files:
///     $ ddata <nuclides> --text  => Ascii tables
///     $ ddata <nuclides> --json  => JSON file
///     $ ddata <nuclides> --mcnp  => MCNP cards
///
///  Sort decay data:
///     $ ddata <nuclides> --sort energy     => Ascending energy
///     $ ddata <nuclides> --sort intensity  => Descending intensity
///
///  Choose radiation type (default: Gamma):
///     $ ddata <nuclides> --rad gamma      => Gamma + X-ray
///     $ ddata <nuclides> --rad xray       => X-ray only
///     $ ddata <nuclides> --rad alpha      => alpha only
///     $ ddata <nuclides> --rad beta-plus  => b+ decay
///     $ ddata <nuclides> --rad beta-minus => b- decay
///     $ ddata <nuclides> --rad electron   => Auger/conversion electron
///
///  Choose output file name:
///     $ ddata <nuclides> --mcnp --text --json --output my_file
///       |_ creates 'my_file.i', 'my_file.txt', 'my_file.json'
///
/// Notes
/// -----
///
/// ! WARNING ! FISPACT-II state notation *should* align with IAEA data
/// (i.e. m,n,o... => m1,m2,m3...), but this is not completely guaranteed.
/// ENSDF can contain much more information than ENDF on structure. Use
/// "m1, m2, m3..." to be explicit.
///
/// Pre-fetched data are from the IAEA chart of nuclides and will generally be
/// up to date and extremely fast. However, '--fetch' can retrieve decay data
/// directly from the IAEA API.
///
/// IAEA records with missing or unobserved intensities are included.
///
/// If your terminal does not support ANSI colour, this can be turned off with
/// the --no-colour option.
#[derive(Parser)]
#[command(
    verbatim_doc_comment,
    arg_required_else_help(true),
    after_help("Note: --help shows more information and examples"),
    term_width(76),
    hide_possible_values(true),
    override_usage("decaydata <nuclides> [options]"),
    styles=custom_style(),
)]
pub struct Cli {
    // * Positional
    /// List of nuclide names
    #[arg(name = "nuclides")]
    pub nuclides: Vec<String>,

    /// Type of decay radiation
    ///
    /// The IAEA chart of nuclides contains the following:
    ///   > Alpha ("a")
    ///   > Beta+ or electron capture ("bp")
    ///   > Beta- ("bm")
    ///   > Gamma decay ("g") [Default]
    ///   > Auger and conversion electron ("e")
    ///   > X-ray ("x")
    #[arg(help_heading("Data options"))]
    #[arg(short, long, value_enum)]
    #[arg(hide_default_value(true))]
    #[arg(default_value_t = CliRadType::Gamma)]
    #[arg(verbatim_doc_comment)]
    #[arg(value_name = "rad")]
    pub rad: CliRadType,

    /// Sort records by property ['energy', 'intensity']
    ///
    /// Defaults to sorting decay data by ascending energy ('e' or 'energy').
    /// Alternatively, data may be sorted in descending order of relative
    /// intensity with 'i' or 'intensity'.
    #[arg(help_heading("Data options"))]
    #[arg(short, long)]
    #[arg(value_name = "property")]
    #[arg(hide_default_value(true))]
    #[arg(default_value = "energy")]
    pub sort: Property,

    /// Query IAEA directly rather than pre-fetched data
    ///
    /// Note that this requires and internet connection and will be much slower
    /// than using pre-processed data.
    #[arg(help_heading("Data options"))]
    #[arg(long)]
    pub fetch: bool,

    /// Prefix for output files
    ///
    /// Defaults to `decay_data`.
    ///
    /// Files are named `<output>.<ext>` and will automatically append the
    /// appropriate extension for the requested file format.
    #[arg(help_heading("Output files"))]
    #[arg(short, long)]
    #[arg(value_name = "name")]
    #[arg(hide_default_value(true))]
    #[arg(default_value = "decay_data")]
    pub output: String,

    /// Text based table
    #[arg(help_heading("Output files"))]
    #[arg(short, long)]
    pub text: bool,

    /// JSON output format
    #[arg(help_heading("Output files"))]
    #[arg(short, long)]
    pub json: bool,

    /// MCNP distribution cards
    #[arg(help_heading("Output files"))]
    #[arg(short, long)]
    pub mcnp: bool,

    /// Starting MCNP distribution number
    ///
    /// Defaults to 100.
    #[arg(help_heading("Output files"))]
    #[arg(short, long)]
    #[arg(value_name = "num")]
    #[arg(hide_default_value(true))]
    #[arg(default_value = "100")]
    pub id: usize,

    /// Fetch raw CSV directly (internet required)
    ///
    /// Quickly request a copy of CSV data directly from the IAEA API.
    ///
    /// Note that these data are completely unaltered, with no post-processing
    /// to fix inconsistencies and other issues with the data they provide.
    #[arg(help_heading("Output files"))]
    #[arg(long)]
    pub csv: bool,

    // * Flags
    /// Verbose logging (-v, -vv)
    ///
    /// If specified, the default log level of INFO is increased to DEBUG (-v)
    /// or TRACE (-vv). Errors and Warnings are always logged unless in quiet
    /// (-q) mode.
    #[arg(short, long)]
    #[arg(action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Supress all log output (overrules --verbose)
    #[arg(short, long)]
    pub quiet: bool,

    /// Turn off table colours
    ///
    /// If your terminal does not support ANSI colour, this can be turned off
    /// with this --no-colour option to remove escape sequences from the stdio.
    #[arg(short, long)]
    pub no_colour: bool,
}

/// Customise the colour styles for clap v4
fn custom_style() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default() | Effects::BOLD)
        .usage(AnsiColor::Cyan.on_default() | Effects::BOLD | Effects::UNDERLINE)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Magenta.on_default())
}

/// Sets up logging at runtime to allow for multiple verbosity levels
pub fn init_logging(cli: &Cli) -> Result<()> {
    let show_level = cli.verbose > 0;

    Ok(stderrlog::new()
        .module("ddata")
        .quiet(cli.quiet)
        .verbosity(cli.verbose as usize + 2)
        .show_level(show_level)
        .color(stderrlog::ColorChoice::Auto)
        .timestamp(stderrlog::Timestamp::Off)
        .init()?)
}
