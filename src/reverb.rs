use std::{fs::File, path::Path};
use std::time::Instant;
use rustfft::FftPlanner;

use wav::BitDepth;

use crate::convolution::rfft_convolve;
use crate::geometry::{ make_box, P, Q, Visible};
use crate::raytracing::profile_room;


pub fn demo(path: &Path) {
    let room = make_box(
        100.,
        100.,
        100.,
        P {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    );
    let room_ref = room.iter().map(|r| r as &dyn Visible).collect();
    let speaker = P {
        x: 50.,
        y: 5.,
        z: 5.,
    };
    let microphone = Q::cw(
        P { x: 48., y: 6., z: 95. },
        P { x: 52., y: 6., z: 95. },
        P { x: 52., y: 4., z: 95. },
        P { x: 48., y: 4., z: 95. }
    );
    let t = Instant::now();
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
    println!("{}", t.elapsed().as_secs_f32());

    let mut in_file = File::open(path).unwrap();
    let (header, data) = wav::read(&mut in_file).unwrap();
    println!("{:?}", header);
    let samples: Vec<f64> = match data {
        BitDepth::Eight(ref _samples) => {_samples.iter().cloned().map(|s| s as f64).collect()},
        BitDepth::Sixteen(ref _samples) => {_samples.iter().cloned().map(|s| s as f64).collect()},
        BitDepth::TwentyFour(ref _samples) => {_samples.iter().cloned().map(|s| s as f64).collect()},
        BitDepth::ThirtyTwoFloat(ref _samples) => {_samples.iter().cloned().map(|s| s as f64).collect()},
        BitDepth::Empty => { panic!("Could not load samples"); }
    };

    fn normalize(xs: &[f64]) -> Vec<f64> {
        let norm = 1. / xs.iter().fold(0., |a: f64, &b| a.max(b));
        xs.iter().map(|x| x * norm).collect()
    }

    let samples = normalize(&samples);
    let kernel = normalize(&kernel);
    let mut planner: FftPlanner<f64> = FftPlanner::new();
    // let reverbed1 = rfft_convolve_real_time(&samples, &kernel, 100000, &planner);
    let reverbed = rfft_convolve(&samples, &kernel, &mut planner);
    // assert!(reverbed1.iter().zip(reverbed.iter()).all(|(a, b)| (a - b).abs() < 1e-7));
    let reverbed = normalize(&reverbed);

    let out_data = match data {
        BitDepth::Eight(_) => {BitDepth::Eight(reverbed.iter().cloned().map(|s| ((s + 0.5) * 128.) as u8).collect())},
        BitDepth::Sixteen(_) => {BitDepth::Sixteen(reverbed.iter().cloned().map(|s| (s * 32767.) as i16).collect())},
        BitDepth::TwentyFour(_) => {BitDepth::TwentyFour(reverbed.iter().cloned().map(|s| (s * 8388607.) as i32).collect())},
        BitDepth::ThirtyTwoFloat(_) => {BitDepth::ThirtyTwoFloat(reverbed.iter().cloned().map(|s| s as f32).collect())},
        BitDepth::Empty => { panic!("Could not load samples"); }
    };

    let mut out_file = File::create("she won't be there_reverb.wav").unwrap();
    wav::write(header, &out_data, &mut out_file).unwrap();
}
