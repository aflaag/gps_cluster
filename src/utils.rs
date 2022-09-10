use std::path::PathBuf;

use geoutils::Location;

const CENTER: Location = Location::new_const(0.0, 0.0);

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
        if self.location == CENTER || self.location.latitude().is_nan() || self.location.longitude().is_nan() { 
            "UNCLASSIFIED".to_string()
        } else {
            let mut output = self.location.latitude().to_string();

            output.push('_');

            output.push_str(&self.location.longitude().to_string());

            output
        }
    }
}
