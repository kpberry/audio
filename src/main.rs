use crate::raytracing::{make_box, profile_room, Visible, L, P, S, T};
use crate::reverb::kernel_reverb;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub mod raytracing;
pub mod reverb;
pub mod tuning;

use reverb::demo;

fn main() {
    demo()
}