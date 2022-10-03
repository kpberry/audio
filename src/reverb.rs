use rustfft::FftPlanner;
use std::time::Instant;
use std::{fs::File, path::Path};

use crate::audio::Audio;
use crate::convolution::rfft_convolve;
use crate::geometry::{make_box, Visible, P, Q};
use crate::raytracing::profile_room;

pub fn demo(path: &Path) {
    let room = make_box(
        10.,
        10.,
        10.,
        P {
            x: 0.,
            y: 0.,
            z: 0.,
        },
    );
    let room_ref = room.iter().map(|r| r as &dyn Visible).collect();
    let speaker = P {
        x: 5.,
        y: 5.,
        z: 1.,
    };
    let microphone = Q::cw(
        P {
            x: 4.8,
            y: 5.2,
            z: 9.,
        },
        P {
            x: 5.2,
            y: 5.2,
            z: 9.,
        },
        P {
            x: 5.2,
            y: 4.8,
            z: 9.,
        },
        P {
            x: 4.8,
            y: 4.8,
            z: 9.,
        },
    );
    let t = Instant::now();
    let kernel = profile_room(
        &room_ref,
        &speaker,
        &microphone,
        100000,
        1000,
        100.,
        343.,
        0.1,
        1000.,
        44100.,
    );
    println!("{}", t.elapsed().as_secs_f32());

    let mut in_file = File::open(path).unwrap();
    let mut audio = Audio::from_wav(&mut in_file).unwrap();

    fn normalize(xs: &[f32]) -> Vec<f32> {
        let norm = 1. / xs.iter().fold(0., |a: f32, &b| a.max(b));
        xs.iter().map(|x| x * norm).collect()
    }

    let samples = normalize(&audio.samples[0]);
    let kernel = normalize(
        &kernel
            .iter()
            .cloned()
            .map(|x| x as f32)
            .collect::<Vec<f32>>(),
    );
    let mut planner: FftPlanner<f32> = FftPlanner::new();
    // let reverbed1 = rfft_convolve_real_time(&samples, &kernel, 100000, &planner);
    let reverbed = rfft_convolve(&samples, &kernel, &mut planner);
    // assert!(reverbed1.iter().zip(reverbed.iter()).all(|(a, b)| (a - b).abs() < 1e-7));
    let reverbed = normalize(&reverbed);

    audio.samples[0] = reverbed.clone();
    audio.samples[1] = reverbed;

    let mut out_file = File::create("data/reverb_out.wav").unwrap();
    audio.to_wav(&mut out_file).unwrap();
}
