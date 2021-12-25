use std::{fs::File, io::Write, path::Path};
use std::cmp::Ordering;
use wav::BitDepth;
use crate::convolution::rfft_convolve;

use crate::raytracing::{make_box, profile_room, Visible, P, S};

pub fn kernel_reverb(samples: &Vec<f64>, kernel: &Vec<f64>) -> Vec<f64> {
    rfft_convolve(samples, kernel)
}


pub fn demo() {
    let room = make_box(
        100.,
        100.,
        100.,
        P {
            x: 0.,
            y: 0.,
            z: 0.,
        },
        true,
    );
    let room_ref = room.iter().map(|r| r as &dyn Visible).collect();
    let speaker = P {
        x: 50.,
        y: 95.,
        z: 5.,
    };
    let microphone = S {
        c: P {
            x: 50.,
            y: 95.,
            z: 95.,
        },
        r: 5.,
    };
    let kernel = profile_room(
        &room_ref,
        &speaker,
        &microphone,
        100000,
        1000,
        10.,
        343.,
        0.05,
        1000.,
        44100.,
    );
    let out_str = format!("{:?}", kernel);
    let path = Path::new("kernel.json");
    let mut file = File::create(path).unwrap();
    file.write_all(out_str.as_bytes());
    // let audio = audio::Audio::read("/home/kpberry/Music/the world is not as large as i thought it was/3.wav");
    // let reverbed = reverb::kernel_reverb(&audio.samples, &kernel);
}
