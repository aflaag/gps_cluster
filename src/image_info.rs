use geoutils::Location;

use std::path::PathBuf;

pub struct ImageInfo {
    path: PathBuf,
    location: Location,
}

impl ImageInfo {
    pub fn new(path: PathBuf, location: Location) -> Self {
        Self { path, location }
    }
}
