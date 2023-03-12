pub mod audio;
mod convolution;
mod filters;
pub mod noise;
mod raytracing;
mod reverb;
mod tuning;

use crate::reverb::demo;
use std::{io, path::Path};

fn main() -> Result<(), io::Error> {
    demo(Path::new("data/hardbass.wav"));
    Ok(())
}
