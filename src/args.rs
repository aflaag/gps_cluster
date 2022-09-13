use clap::Parser;
use std::path::PathBuf;

/// This program is able to create folders of pictures,
/// grouping them by their GPS position.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ProgramArgs {
    /// The input file
    #[clap(short, long, value_parser)]
    pub input: PathBuf,

    /// The output file
    #[clap(short, long, value_parser)]
    pub output: PathBuf,

    /// The radius used to generate clusters
    #[clap(short, long, value_parser)]
    pub threshold: f64,

    #[clap(long, value_parser, requires = "time")]
    pub relocate: bool,

    #[clap(long, value_parser)]
    pub time: Option<u64>,

    #[clap(long, value_parser, requires = "api-key")]
    pub human_readable: bool,

    #[clap(long, value_parser)]
    pub api_key: Option<String>,

    /// Use verbose output
    #[clap(short, long, value_parser)]
    pub verbose: bool,
}
