use gps_cluster::{features::*, cluster::{Cluster, Image}, args::ProgramArgs};

use std::{path::PathBuf, fs::{File, metadata, read_dir, create_dir, copy}, time::Duration, io::BufReader};
use clap::Parser;
use exif::{Tag, In, Value, Reader};
use geoutils::{Location, Distance};

fn main() {
    let mut args = ProgramArgs::parse();

    let mut image_clusters = Vec::new();

    let out = read_dir(&args.output);

    if !metadata(&args.input).unwrap().is_dir() {
        eprintln!("Error: the input path must be a folder, not a file.");
    } else if out.is_ok() {
        if out.unwrap().next().is_some() {
            eprintln!("Error: the given output folder is not empty.")
        } else {
            let mut unclassified_cluster = Cluster::default();

            generate_clusters(&args, &mut image_clusters, &mut unclassified_cluster);

            if args.relocate {
                relocate(&mut image_clusters, &mut unclassified_cluster, args.time.unwrap(), args.verbose);
            }

            image_clusters.push(unclassified_cluster);

            create_dirs(&image_clusters, &mut args.output, args.verbose);
        }
    } else {
        eprintln!("Error: the output path doesn't exist, or it's not a folder.")
    }
}
