use crate::{cluster::{Cluster, Image}, args::ProgramArgs};

use std::{path::PathBuf, fs::{File, metadata, read_dir, create_dir, copy}, io::BufReader, cmp::Ordering};
use exif::{Tag, In, Value, Reader};
use geoutils::{Location, Distance};
use clap::Parser;

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

