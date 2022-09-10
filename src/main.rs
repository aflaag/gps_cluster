use gps_cluster::image_info::ImageInfo;

use std::{io::{Read, Seek, SeekFrom}, path::{Path, PathBuf}, fs::{File, metadata, read_dir}};
use clap::Parser;

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
}

fn walk(input: &PathBuf, image_infos: &mut Vec<ImageInfo>) {
    if metadata(input).unwrap().is_file() {
        match rexif::parse_file(input) {
            Ok(exif) => {
                if let Some(latitude) = &exif.entries.iter().find(|entry| entry.tag == rexif::ExifTag::GPSLatitude) {
                    if let Some(longitude) = &exif.entries.iter().find(|entry| entry.tag == rexif::ExifTag::GPSLongitude) {
                        println!("{}, {},{}", input.display(), latitude.value_more_readable, longitude.value_more_readable);
                    } else {
                        println!("Error in {:?}: longitude not found", input);
                    }
                } else {
                    println!("Error in {:?}: latitude not found", input);
                }
            },
            Err(e) => {
                println!("Error in {:?}: {}", input, e)
            }
        }
    } else {
        for path in read_dir(input).unwrap() {
            walk(&path.unwrap().path(), image_infos);
        }
    }
}

fn main() {
    let args = Args::parse();

    let mut image_infos = Vec::new();

    if !metadata(&args.input).unwrap().is_dir() {
        eprintln!("Error: the input path must be a folder, not a file.");
    } else if read_dir(&args.output).is_ok() {
        walk(&args.input, &mut image_infos);
    } else {
        eprintln!("Error: the output path doesn't exist, or it's not a folder.")
    }
}
