use std::path::PathBuf;

use geoutils::Location;

pub fn dms_to_dd(deg: f64, min: f64, sec: f64) -> f64 {
    deg + min / 60.0 + sec / 3600.0
}

#[derive(Debug, Clone)]
pub struct Cluster {
    pub location: Location,
    pub images: Vec<PathBuf>,
}

impl Cluster {
    pub fn fmt_location(&self) -> String {
        let mut output = self.location.latitude().to_string();

        output.push('_');

        output.push_str(&self.location.longitude().to_string());

        output
    }
}
