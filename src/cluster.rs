use crate::utils::CENTER;

use std::path::PathBuf;
use geoutils::Location;
use chrono::NaiveDateTime;
use exif::{Tag, In, Value, Exif};

/// Used to store a single image cluster.
#[derive(Debug, Clone)]
pub struct Cluster {
    /// One of the images' location.
    pub location: Location,

    /// The images.
    pub images: Vec<Image>,
}

impl Cluster {
    /// Returns `false` if the location is `utils::CENTER`,
    /// or if the coordinates are NaN.
    pub fn is_classified(&self) -> bool {
        self.location != CENTER && !self.location.latitude().is_nan() && !self.location.longitude().is_nan()
    }

    /// Used to evaluate the name of the folder of the cluster,
    /// based on the coordinates of the location; returns `UNCLASSIFIED`
    /// when the cluster has an invalid location.
    pub fn fmt_location(&self) -> String {
        if !self.is_classified() {
            "UNCLASSIFIED".to_string()
        } else {
            let mut output = self.location.latitude().to_string();

            output.push('_');

            output.push_str(&self.location.longitude().to_string());

            output
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
        if let Some(datetime) = exif.get_field(Tag::DateTime, In::PRIMARY) {
            self.update_timestamp_internals(&datetime.value)
        } else if let Some(datetime_original) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
            self.update_timestamp_internals(&datetime_original.value)
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
