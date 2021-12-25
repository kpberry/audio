use crate::raytracing::{make_box, profile_room, Visible, L, P, S, T};
use crate::reverb::kernel_reverb;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

mod raytracing;
mod reverb;
mod tuning;
mod convolution;

use reverb::demo;

fn main() {
    demo()
}