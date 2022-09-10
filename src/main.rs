use gps_cluster::utils::{self, Cluster};

use std::{path::PathBuf, fs::{metadata, read_dir, create_dir, copy}};
use clap::Parser;
use exif::{Tag, In, Value};
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

    /// Use verbose output
    #[clap(short, long, value_parser)]
    verbose: bool,
}

fn walk(input: &PathBuf, image_clusters: &mut Vec<Cluster>, threshold: Distance, verbose: bool) {
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

                                let location = Location::new(lat_dd, lon_dd);

                                let mut found_cluster = false;

                                for cluster in image_clusters.iter_mut() {
                                    match location.is_in_circle(&cluster.location, threshold) {
                                        Ok(is_in_circle) => {
                                            if is_in_circle {
                                                cluster.images.push(input.clone());

                                                found_cluster = true;

                                                break;
                                            }
                                        },
                                        Err(e) => eprintln!("Error: {}", e),
                                    }
                                }

                                if !found_cluster {
                                    image_clusters.push(Cluster {
                                        location,
                                        images: vec![input.clone()],
                                    })
                                }
                            }
                        } else {
                            if verbose {
                                eprintln!("Ignoring {:?}: longitude not found.", input);
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
            walk(&path.unwrap().path(), image_clusters, threshold, verbose);
        }
    }
}

fn create_dirs(image_clusters: &Vec<Cluster>, output: &mut PathBuf, verbose: bool) {
    image_clusters
        .iter()
        .for_each(|cluster| {
            output.push(cluster.fmt_location());

            if let Ok(exists) = output.try_exists() {
                let mut proceed = true;

                if !exists {
                    if create_dir(&output).is_err() {
                        proceed = false;

                        if verbose {
                            eprintln!("Error: an error occured while trying to create {:?} directory.", output);
                        }
                    }
                }

                if proceed {
                    cluster
                        .images
                        .iter()
                        .for_each(|path| {
                            output.push(path.file_name().unwrap());

                            if copy(path, &output).is_err() {
                                eprintln!("Error: an error occured while trying to save {:?}.", output);
                            } else {
                                if verbose {
                                    println!("Successfully saved {:?}.", output);
                                }
                            }

                            output.pop();
                        })
                }
            } else {
                if verbose {
                    eprintln!("Error: can't check existence of {:?}", output);
                }
            }

            output.pop();
        })
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
            walk(&args.input, &mut image_clusters, Distance::from_meters(args.threshold), args.verbose);

            create_dirs(&image_clusters, &mut args.output, args.verbose);
        }
    } else {
        eprintln!("Error: the output path doesn't exist, or it's not a folder.")
    }
}
