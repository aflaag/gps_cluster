use gps_cluster::{utils, cluster::{Cluster, Image}};

use std::{path::PathBuf, fs::{File, metadata, read_dir, create_dir, copy}, time::Duration, io::BufReader};
use clap::Parser;
use exif::{Tag, In, Value, Reader};
use geoutils::{Location, Distance};

/// This program is able to create folders of pictures,
/// grouping them by their GPS position.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The input file
    #[clap(short, long, value_parser)]
    input: PathBuf,

    /// The output file
    #[clap(short, long, value_parser)]
    output: PathBuf,

    /// The radius used to generate clusters
    #[clap(short, long, value_parser)]
    threshold: f64,

    /// If enabled, the program tries to guess the location
    /// of the images which don't have a valid metadata.
    #[clap(long, value_parser, requires = "time")]
    try_guess: bool,

    #[clap(long, value_parser)]
    time: Option<u64>,

    /// Use verbose output
    #[clap(short, long, value_parser)]
    verbose: bool,
}

fn main() {
    let mut args = Args::parse();

    let mut image_clusters = Vec::new();

    let out = read_dir(&args.output);

    if !metadata(&args.input).unwrap().is_dir() {
        eprintln!("Error: the input path must be a folder, not a file.");
    } else if out.is_ok() {
        if out.unwrap().next().is_some() {
            eprintln!("Error: the given output folder is not empty.")
        } else {
            utils::generate_clusters(&args.input, &mut image_clusters, Distance::from_meters(args.threshold), args.verbose);

            if args.verbose {
                println!("Clusters found before merging: {}", image_clusters.len());
            }

            utils::merge_clusters(&mut image_clusters);

            if args.verbose {
                println!("Clusters found after merging: {}", image_clusters.len());
            }

            if args.try_guess {
                utils::try_guess(&mut image_clusters, args.time.unwrap(), args.verbose);
            }

            utils::create_dirs(&image_clusters, &mut args.output, args.verbose);
        }
    } else {
        eprintln!("Error: the output path doesn't exist, or it's not a folder.")
    }
}
