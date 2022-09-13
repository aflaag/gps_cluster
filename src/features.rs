use crate::{cluster::{Cluster, Image}, args::ProgramArgs, utils::*};

use std::{path::PathBuf, fs::{File, metadata, read_dir, create_dir, copy}, io::BufReader, cmp::Ordering};
use exif::{Tag, In, Value, Reader};
use geoutils::{Location, Distance};

fn parse_file(input: &PathBuf, image_clusters: &mut Vec<Cluster>, unclassified_cluster: &mut Cluster, threshold: Distance, verbose: bool) {
    let mut bufreader = BufReader::new(File::open(input).unwrap());

    let mut image = Image {
        path: input.clone(),
        timestamp: None,
        location: None,
    };

    if let Ok(exif) = Reader::new().read_from_container(&mut bufreader) {
        image.update_timestamp(&exif);

        if let Some(latitude) = exif.get_field(Tag::GPSLatitude, In::PRIMARY) {
            let lat_dd: DecimalDegrees = DMS::from(&latitude.value).into();

            if let Some(longitude) = exif.get_field(Tag::GPSLongitude, In::PRIMARY) {
                let lon_dd: DecimalDegrees = DMS::from(&longitude.value).into();

                if verbose {
                    println!("Found position for {:?}: ({}, {})", input, lat_dd.0, lon_dd.0);
                }

                image.location = Some(Location::new(lat_dd.0, lon_dd.0));

                if !image.is_classifiable() {
                    unclassified_cluster.images.push(image);
                    return;
                }

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
            } else {
                unclassified_cluster.images.push(image);
            }
        } else {
            unclassified_cluster.images.push(image);
        }
    } else if verbose {
        eprintln!("Ignoring {:?}: unknown file format.", input);
    }
}

fn generate_clusters_internals(input: &PathBuf, image_clusters: &mut Vec<Cluster>, unclassified_cluster: &mut Cluster, threshold: Distance, human_readable: bool, verbose: bool) {
    if metadata(input).unwrap().is_file() {
        parse_file(input, image_clusters, unclassified_cluster, threshold, verbose);
    } else {
        for path in read_dir(input).unwrap() {
            generate_clusters_internals(&path.unwrap().path(), image_clusters, unclassified_cluster, threshold, human_readable, verbose);
        }
    }
}

pub fn generate_clusters(args: &ProgramArgs, image_clusters: &mut Vec<Cluster>, unclassified_cluster: &mut Cluster) {
    generate_clusters_internals(&args.input, image_clusters, unclassified_cluster, Distance::from_meters(args.threshold), args.human_readable, args.verbose)
}

pub fn relocate(image_clusters: &mut [Cluster], unclassified_cluster: &mut Cluster, time: i64, verbose: bool) {
    let mut to_remove = Vec::new();

    unclassified_cluster
        .images
        .iter()
        .enumerate()
        .filter(|(_, image)| image.timestamp.is_some())
        .for_each(|(idx, unclassified_image)| {
            let unclassified_image_timestamp = unclassified_image.timestamp.unwrap();

            let best_cluster_and_ratio = image_clusters
                .iter_mut()
                .map(|cluster| (cluster.reliability(time, unclassified_image_timestamp), cluster))
                .max_by(|(ratio1, _), (ratio2, _)| ratio1.partial_cmp(ratio2).unwrap_or(Ordering::Equal));

            if let Some((ratio, cluster)) = best_cluster_and_ratio {
                if (ratio - 0.0).abs() >= f32::EPSILON {
                    cluster.images.push(unclassified_image.clone());

                    to_remove.push(idx);

                    if verbose {
                        println!("{:?} relocated into {:?} with a {}% reliability.", unclassified_image.path, cluster.fmt_location(), ratio * 100.0);
                    }
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
            } else {
                eprintln!("Error: can't check the existence of {:?}", output);
            }

            output.pop();
        })
}
