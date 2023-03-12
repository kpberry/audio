use parry3d::bounding_volume::Aabb;
use parry3d::math::Point;
use parry3d::query::RayCast;
use rustfft::FftPlanner;
use std::time::Instant;
use std::{fs::File, path::Path};

use crate::audio::Audio;
use crate::convolution::rfft_convolve;
use crate::raytracing::profile_room;

pub fn demo(path: &Path) {
    let room = Aabb::new(Point::new(0.0, 0.0, 0.0), Point::new(10.0, 10.0, 100.0));
    let speaker = Point::new(5.0, 5.0, 1.0);
    let microphone = Aabb::new(Point::new(4.9, 4.9, 99.0), Point::new(5.1, 5.1, 99.1));

    let mut geometry: Vec<&dyn RayCast> = Vec::new();
    geometry.push(&room);

    let t = Instant::now();
    let kernel = profile_room(
        &geometry,
        &speaker,
        &microphone,
        1000,
        1000,
        100.,
        343.,
        0.01,
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
    let reverbed = rfft_convolve(&samples, &kernel, &mut planner);
    let reverbed = normalize(&reverbed);

    audio.samples[0] = reverbed.clone();
    audio.samples[1] = reverbed;

    let mut out_file = File::create("data/reverb_out.wav").unwrap();
    audio.to_wav(&mut out_file).unwrap();
}
