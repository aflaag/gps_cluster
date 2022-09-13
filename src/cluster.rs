use crate::utils::{CENTER, DecimalDegrees, DMS};

use std::path::PathBuf;
use geoutils::Location;
use chrono::NaiveDateTime;
use exif::{Tag, In, Value, Exif};
use google_maps::prelude::*;
use google_maps::geocoding::{response::Response, error::Error};

/// Used to store a single image cluster.
#[derive(Debug, Clone)]
pub struct Cluster {
    /// One of the images' location.
    pub location: Location,

    /// The images.
    pub images: Vec<Image>,

    pub location_string: Option<String>,
}

impl Cluster {
    /// Returns `false` if the location is `utils::CENTER`,
    /// or if the coordinates are NaN.
    pub fn is_classified(&self) -> bool {
        self.location != CENTER && !self.location.latitude().is_nan() && !self.location.longitude().is_nan()
    }

    pub async fn reverse_geocoding(&self, client: &GoogleMapsClient) -> Option<GeocodingResponse> {
        for image in &self.images {
            if let Ok(position) = LatLng::try_from_f64(image.location.unwrap().latitude(), image.location.unwrap().longitude()) {
                let request = client
                    .reverse_geocoding(position)
                    .with_result_type(PlaceType::PointOfInterest)
                    // .with_result_type(PlaceType::Locality)
                    // .with_result_type(PlaceType::Cities)
                    .execute()
                    .await;
                
                if let Ok(content) = request {
                    return Some(content);
                }
            }
        }

        None
    }

    unsafe fn update_location_with_coordinates_unchecked(&mut self) {
        let mut location_string = self.location.latitude().to_string();

        location_string.push('_');

        location_string.push_str(&self.location.longitude().to_string());

        self.location_string = Some(location_string);
    }

    pub async fn update_location(&mut self, gm_client: &Option<GoogleMapsClient>) {
        if self.location_string.is_none() {
            if !self.is_classified() {
                self.location_string = Some("UNCLASSIFIED".to_string());
            } else {
                if let Some(client) = gm_client {
                    if let Some(content) = self.reverse_geocoding(&client).await {
                        self.location_string = Some(
                            content
                                .results
                                .iter()
                                .flat_map(|result| {
                                    result
                                        .address_components
                                        .iter()
                                        .map(|address_component| {
                                            let mut string = address_component.short_name.to_string();

                                            string.push(' ');

                                            string
                                        })
                                })
                                .collect::<Vec<String>>()
                                .join(" - ")
                        );
                    } else {
                        unsafe { self.update_location_with_coordinates_unchecked() }
                    }
                } else {
                    unsafe { self.update_location_with_coordinates_unchecked() }
                }
            }
        }
    }

    pub fn reliability(&self, time: i64, unclassified_image_timestamp: NaiveDateTime) -> f32 {
        self
            .images
            .iter()
            .filter(|image| {
                image.timestamp.is_some() &&
                image.is_classifiable() &&
                (image.timestamp.unwrap() - unclassified_image_timestamp).num_seconds().abs() < time
            })
            .count() as f32 / self.images.len() as f32
    }
}

impl Default for Cluster {
    fn default() -> Self {
        Cluster {
            location: CENTER,
            images: vec![],
            location_string: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Image {
    pub path: PathBuf,
    pub timestamp: Option<NaiveDateTime>,
    pub location: Option<Location>,
}

impl Image {
    fn update_timestamp_internals(&mut self, time: &Value) {
        if let Value::Ascii(timestamp) = time {
            self.timestamp = Some(
                NaiveDateTime::parse_from_str(
                    &timestamp[0].iter().map(|byte| *byte as char).collect::<String>(),
                    "%Y:%m:%d %H:%M:%S"
                )
                .unwrap()
            );
        }
    }

    pub fn update_timestamp(&mut self, exif: &Exif) {
        if self.timestamp.is_none() {
            if let Some(datetime) = exif.get_field(Tag::DateTime, In::PRIMARY) {
                self.update_timestamp_internals(&datetime.value)
            } else if let Some(datetime_original) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
                self.update_timestamp_internals(&datetime_original.value)
            }
        }
    }

    pub fn update_location(&mut self, exif: &Exif) {
        if self.location.is_none() {
            if let Some(latitude) = exif.get_field(Tag::GPSLatitude, In::PRIMARY) {
                let lat_dd: DecimalDegrees = DMS::from(&latitude.value).into();

                if let Some(longitude) = exif.get_field(Tag::GPSLongitude, In::PRIMARY) {
                    let lon_dd: DecimalDegrees = DMS::from(&longitude.value).into();

                    self.location = Some(Location::new(lat_dd.0, lon_dd.0));
                }
            }
        }
    }

    pub fn is_classifiable(&self) -> bool {
        if let Some(loc) = self.location {
            loc != CENTER && !loc.latitude().is_nan() && !loc.longitude().is_nan()
        } else {
            false
        }
    }
}
