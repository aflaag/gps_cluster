use gps_cluster::{image_info::ImageInfo, utils};

use std::{io::{Read, Seek, SeekFrom}, path::{Path, PathBuf}, fs::{File, metadata, read_dir}};
use clap::Parser;
use exif::{Tag, In, Value};

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

    #[clap(short, long, value_parser)]
    threshold: f32,

    /// Use verbose output
    #[clap(short, long, value_parser)]
    verbose: bool,
}

fn walk(input: &PathBuf, image_infos: &mut Vec<ImageInfo>, verbose: bool) {
    if metadata(input).unwrap().is_file() {
        let file = std::fs::File::open(input).unwrap();
        let mut bufreader = std::io::BufReader::new(&file);
        let exifreader = exif::Reader::new();

        if let Ok(exif) = exifreader.read_from_container(&mut bufreader) {
                if let Some(latitude) = exif.get_field(Tag::GPSLatitude, In::PRIMARY) {
                    if let Value::Rational(rationals) = &latitude.value {
                        let lat_deg = rationals[0].to_f64();
                        let lat_min = rationals[1].to_f64();
                        let lat_sec = rationals[2].to_f64();

                        let lat_dd = utils::dms_to_dd(lat_deg, lat_min, lat_sec);

                        if let Some(longitude) = exif.get_field(Tag::GPSLongitude, In::PRIMARY) {
                            if let Value::Rational(rationals) = &longitude.value {
                                let lon_deg = rationals[0].to_f64();
                                let lon_min = rationals[1].to_f64();
                                let lon_sec = rationals[2].to_f64();

                                let lon_dd = utils::dms_to_dd(lon_deg, lon_min, lon_sec);


                                if verbose {
                                    println!("{:?}: ({}, {})", input, lat_dd, lon_dd)
                                }
                            }
                        }
                    }
                } else {
                    if verbose {
                        eprintln!("Ignoring {:?}: latitude not found.", input);
                    }
                }
        } else {
            if verbose {
                eprintln!("Ignoring {:?}: unknown file format.", input);
            }
        }
    } else {
        for path in read_dir(input).unwrap() {
            walk(&path.unwrap().path(), image_infos, verbose);
        }
    }
}

fn main() {
    let args = Args::parse();

    let mut image_infos = Vec::new();

    if !metadata(&args.input).unwrap().is_dir() {
        eprintln!("Error: the input path must be a folder, not a file.");
    } else if read_dir(&args.output).is_ok() {
        walk(&args.input, &mut image_infos, args.verbose);
    } else {
        eprintln!("Error: the output path doesn't exist, or it's not a folder.")
    }
}
