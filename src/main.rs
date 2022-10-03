use std::{path::Path, fs::File};

use crate::audio::Audio;

pub mod audio;


fn main() {
    let mut file = File::open(Path::new("data/3.wav")).unwrap();
    let mut out_file = File::create("data/3_test.wav").unwrap();
    let audio = Audio::from_wav(&mut file).unwrap();
    audio.to_wav(&mut out_file).unwrap();
    println!("done?");
}