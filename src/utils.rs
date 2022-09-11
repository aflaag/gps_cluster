use crate::cluster::{Cluster, Image};

use std::{path::PathBuf, fs::{File, metadata, read_dir, create_dir, copy}, io::BufReader};
use exif::{Tag, In, Value, Reader};
use geoutils::{Location, Distance};
use chrono::{NaiveDateTime, Duration};

/// Represents the center of the axes, the point of intersection
/// between the prime meridian (Greenwich) and the Equator.
pub const CENTER: Location = Location::new_const(0.0, 0.0);

pub struct DMS(pub f64, pub f64, pub f64);

impl From<&Value> for DMS {
    fn from(value: &Value) -> Self {
        if let Value::Rational(rationals) = value {
            Self(rationals[0].to_f64(), rationals[1].to_f64(), rationals[2].to_f64())
        } else {
            panic!("Error: invalid Value variant.")
        }
    }
}

pub struct DecimalDegrees(pub f64);

impl From<DMS> for DecimalDegrees {
    fn from(dms: DMS) -> Self {
        Self(dms.0 + dms.1 / 60.0 + dms.2 / 3600.0)
    }
}

pub fn parse_file(input: &PathBuf, image_clusters: &mut Vec<Cluster>, threshold: Distance, verbose: bool) {
    let mut bufreader = BufReader::new(File::open(input).unwrap());

    if let Ok(exif) = Reader::new().read_from_container(&mut bufreader) {
        let mut image = Image {
            path: input.clone(),
            timestamp: None,
            location: None,
        };

        if let Some(time) = exif.get_field(Tag::DateTime, In::PRIMARY) {
            if let Value::Ascii(timestamp) = &time.value {
                image.timestamp = Some(
                    NaiveDateTime::parse_from_str(
                        &timestamp[0].iter().map(|byte| *byte as char).collect::<String>(),
                        "%Y:%m:%d %H:%M:%S"
                    )
                    .unwrap()
                );
            }
        }

        if let Some(latitude) = exif.get_field(Tag::GPSLatitude, In::PRIMARY) {
            let lat_dd: DecimalDegrees = DMS::from(&latitude.value).into();

            if let Some(longitude) = exif.get_field(Tag::GPSLongitude, In::PRIMARY) {
                let lon_dd: DecimalDegrees = DMS::from(&longitude.value).into();

                if verbose {
                    println!("{:?}: ({}, {})", input, lat_dd.0, lon_dd.0);
                }

                image.location = Some(Location::new(lat_dd.0, lon_dd.0));

                let mut found_cluster = false;

                for cluster in image_clusters.iter_mut() {
                    match image.location.unwrap().is_in_circle(&cluster.location, threshold) {
                        Ok(is_in_circle) => {
                            if is_in_circle {
                                cluster.images.push(image.clone());

                                found_cluster = true;

                                break;
                            }
                        },
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }

                if !found_cluster {
                    image_clusters.push(Cluster {
                        location: image.location.unwrap(),
                        images: vec![image],
                    })
                }
            } else if verbose {
                eprintln!("Ignoring {:?}: longitude not found.", input);
            }
        } else if verbose {
            eprintln!("Ignoring {:?}: latitude not found.", input);
        }
    } else if verbose {
        eprintln!("Ignoring {:?}: unknown file format.", input);
    }
}

pub fn generate_clusters(input: &PathBuf, image_clusters: &mut Vec<Cluster>, threshold: Distance, verbose: bool) {
    if metadata(input).unwrap().is_file() {
        parse_file(input, image_clusters, threshold, verbose);
    } else {
        for path in read_dir(input).unwrap() {
            generate_clusters(&path.unwrap().path(), image_clusters, threshold, verbose);
        }
    }
}

pub fn merge_unclassified(image_clusters: &mut Vec<Cluster>) -> Cluster {
    let mut to_remove = Vec::new();

    let mut unclassified_cluster = Cluster {
        location: CENTER,
        images: Vec::new(),
    };

    image_clusters
        .iter()
        .enumerate()
        .for_each(|(idx, cluster)| {
            if !cluster.is_classified() {
                unclassified_cluster.images.extend(cluster.images.clone());

                to_remove.push(idx);
            }
        });

    to_remove
        .iter()
        .enumerate()
        .for_each(|(offset, idx)| { image_clusters.remove(idx - offset); });

    unclassified_cluster
}

pub fn try_guess(image_clusters: &mut Vec<Cluster>, unclassified_cluster: &mut Cluster, time: i64, verbose: bool) {
    let dur_time = Duration::seconds(time.try_into().unwrap());

    let mut to_remove = Vec::new();

    unclassified_cluster
        .images
        .iter()
        .enumerate()
        .filter(|(_, image)| image.timestamp.is_some())
        .for_each(|(idx, unclassified_image)| {
            for cluster in image_clusters.iter_mut() {
                if cluster
                    .images
                    .iter()
                    .filter(|image| image.timestamp.is_some())
                    .any(|image| image.timestamp.unwrap() - unclassified_image.timestamp.unwrap() < dur_time)
                {
                    cluster.images.push(unclassified_image.clone());
                    to_remove.push(idx);

                    break;
                }
            }
        });

    to_remove
        .iter()
        .enumerate()
        .for_each(|(offset, idx)| { unclassified_cluster.images.remove(idx - offset); });
}

pub fn create_dirs(image_clusters: &[Cluster], output: &mut PathBuf, verbose: bool) {
    image_clusters
        .iter()
        .for_each(|cluster| {
            output.push(cluster.fmt_location());

            if let Ok(exists) = output.try_exists() {
                if !exists && create_dir(&output).is_err() {
                    eprintln!("Error: an error occured while trying to create {:?} directory.", output);
                } else {
                    cluster
                        .images
                        .iter()
                        .for_each(|image| {
                            output.push(image.path.file_name().unwrap());

                            if copy(&image.path, &output).is_err() {
                                eprintln!("Error: an error occured while trying to save {:?}.", output);
                            } else if verbose {
                                println!("Successfully saved {:?}.", output);
                            }

                            output.pop();
                        })
                }
            } else if verbose {
                eprintln!("Error: can't check existence of {:?}", output);
            }

            output.pop();
        })
}
