use std::f64::consts::PI;
use rand::Rng;
use crate::geometry::{L, P, S, Visible};

pub fn forward_ray_trace(
    r: &L,
    geometry: &Vec<&dyn Visible>,
    target: &dyn Visible,
    hits: &mut Vec<(P, f64, usize)>,
    bounces: usize,
    max_bounces: usize,
    distance: f64,
    max_distance: f64,
) {
    if distance > max_distance || bounces > max_bounces {
        return;
    }

    let mut closest: Option<&dyn Visible> = None;
    let mut closest_distance: f64 = f64::MAX;
    for obj in geometry {
        if let Some(intersection) = obj.ray_intersection(&r) {
            let intersection_distance = r.a.dist(&intersection);
            if 1e-7 < intersection_distance && intersection_distance < closest_distance {
                closest = Some(*obj);
                closest_distance = intersection_distance;
            }
        }
    }

    if let Some(hit) = target.ray_intersection(&r) {
        let hit_dist = r.a.dist(&hit);
        if hit_dist <= closest_distance {
            // TODO it seems like this should be distance + hit_dist, but that causes a weird
            //  phasing effect for some reason
            hits.push((hit, distance + closest_distance + hit_dist, bounces));
        }
    }

    if let Some(obj) = closest {
        if let Some(reflection) = obj.ray_reflection(&r) {
            forward_ray_trace(&reflection, geometry, target, hits,
                              bounces + 1, max_bounces,
                              distance + closest_distance, max_distance);
        };
    }
}

pub fn profile_room(
    room: &Vec<&dyn Visible>,
    speaker: &P,
    microphone: &dyn Visible,
    samples: usize,
    max_bounces: usize,
    max_delay: f64,
    speed_of_sound: f64,
    decay: f64,
    base_impulse: f64,
    sample_rate: f64,
) -> Vec<f64> {
    let mut rng = rand::thread_rng();

    let mut hits = Vec::new();
    let max_distance = max_delay * speed_of_sound;
    let inv_speed_of_sound = 1.0 / speed_of_sound;

    for s in 0..=samples {
        if s % 1000 == 0 {
            println!("{}/{}", s, samples);
        }
        let u: f64 = rng.gen_range(-1.0..1.0);
        let t: f64 = rng.gen_range(0.0..PI);

        let x = (1. - u * u) * t.cos();
        let y = (1. - u * u) * t.sin();
        let z = u;

        let p = P { x, y, z };
        let r = L {
            a: speaker.clone(),
            b: speaker + &p,
        };

        forward_ray_trace(&r, room, microphone, &mut hits,
                          0, max_bounces, 0., max_distance);
    }

    let sample_timings: Vec<usize> = hits
        .iter()
        .map(|(_, d, _)| (sample_rate * d * inv_speed_of_sound).round() as usize)
        .collect();
    // microphones measure sound pressure, which decays linearly with distance
    let pressures: Vec<f64> = hits
        .iter()
        .map(|(_, d, b)| base_impulse / d * (1. - decay).powi(*b as i32))
        .collect();
    let max_timing: usize = sample_timings.iter().cloned().fold(0, usize::max) + 1;
    let mut kernel = vec![0.; max_timing];
    sample_timings
        .iter()
        .zip(pressures)
        .for_each(|(t, e)| kernel[*t] += e);
    let max_k = kernel.iter().cloned().fold(1., f64::max);
    let kernel = kernel.iter().map(|k| k / max_k).collect();
    kernel
}
