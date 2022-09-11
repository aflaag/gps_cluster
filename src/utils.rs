use std::path::PathBuf;
use geoutils::Location;

/// Represents the center of the axes, the point of intersection
/// between the prime meridian (Greenwich) and the Equator.
const CENTER: Location = Location::new_const(0.0, 0.0);

/// Converts a DMS coordinate into a DD coordinate.
pub fn dms_to_dd(deg: f64, min: f64, sec: f64) -> f64 {
    deg + min / 60.0 + sec / 3600.0
}

/// Used to store a single image cluster.
#[derive(Debug, Clone)]
pub struct Cluster {
    /// One of the images' location.
    pub location: Location,

    /// The paths of the images.
    pub images: Vec<PathBuf>,
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
}
