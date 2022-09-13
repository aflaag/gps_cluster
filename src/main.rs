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

    #[clap(long, value_parser, requires = "time")]
    relocate: bool,

    #[clap(long, value_parser)]
    time: Option<i64>,

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
            let mut unclassified_cluster = Cluster::default();

            utils::generate_clusters(&args.input, &mut image_clusters, &mut unclassified_cluster, Distance::from_meters(args.threshold), args.verbose);

            if args.relocate {
                utils::relocate(&mut image_clusters, &mut unclassified_cluster, args.time.unwrap(), args.verbose);
            }

            image_clusters.push(unclassified_cluster);

            utils::create_dirs(&image_clusters, &mut args.output, args.verbose);
        }
    } else {
        eprintln!("Error: the output path doesn't exist, or it's not a folder.")
    }
}
