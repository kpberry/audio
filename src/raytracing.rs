use parry3d::{
    math::Real,
    na::{Point3, Vector3},
    query::{Ray, RayCast},
};
use rand::{rngs::ThreadRng, Rng};
use std::f64::consts::PI;

fn proj(u: &Vector3<Real>, v: &Vector3<Real>) -> Vector3<Real> {
    (v / v.norm()) * (u.dot(v))
}

pub fn forward_ray_trace(
    ray: &Ray,
    geometry: &Vec<&dyn RayCast>,
    bounces: usize,
    max_bounces: usize,
    distance: f32,
    max_distance: f32,
) -> Vec<(usize, f32, usize)> {
    if distance > max_distance || bounces >= max_bounces {
        return Vec::new();
    }

    let mut closest = None;
    let mut closest_distance: f32 = f32::MAX;
    for (i, obj) in geometry.iter().enumerate() {
        if let Some(intersection) = obj.cast_local_ray_and_get_normal(&ray, max_distance, false) {
            if intersection.toi < closest_distance {
                closest = Some((i, intersection));
                closest_distance = intersection.toi;
            }
        }
    }

    if let Some((i, intersection)) = closest {
        let intersection_point: Point3<_> = ray.origin + ray.dir * intersection.toi;
        let normal_ray = Ray::new(intersection_point, intersection.normal);
        let reflection_dir: Vector3<_> = ray.dir - proj(&ray.dir, &normal_ray.dir) * 2.0;
        // reflection needs to be slightly offset from its surface so that it's not incorrectly occluded
        let reflection_origin = intersection_point + reflection_dir * 1e-5;
        let reflection = Ray::new(reflection_origin, reflection_dir);
        let mut hits = forward_ray_trace(
            &reflection,
            geometry,
            bounces + 1,
            max_bounces,
            distance + closest_distance,
            max_distance,
        );
        hits.push((i, distance + closest_distance, bounces));
        hits
    } else {
        Vec::new()
    }
}

pub fn random_spherical_direction(rng: &mut ThreadRng) -> Vector3<f32> {
    let u: f32 = rng.gen_range(-1.0..1.0);
    let t: f32 = rng.gen_range(0.0..PI as f32);

    let x = (1. - u * u) * t.cos();
    let y = (1. - u * u) * t.sin();
    let z = u;

    Vector3::new(x, y, z)
}

pub fn profile_room(
    room: &Vec<&dyn RayCast>,
    speaker: &Point3<f32>,
    microphone: &dyn RayCast,
    samples: usize,
    max_bounces: usize,
    max_delay: f32,
    speed_of_sound: f32,
    decay: f32,
    base_impulse: f32,
    sample_rate: f32,
) -> Vec<f32> {
    let mut rng = rand::thread_rng();

    let max_distance = max_delay * speed_of_sound;

    let mut geometry = room.clone();
    geometry.push(microphone);

    let mut microphone_hits: Vec<(f32, usize)> = Vec::new();
    let microphone_index = geometry.len() - 1;

    for s in 0..=samples {
        if s % 1000 == 0 {
            println!("{}/{}", s, samples);
        }

        let dir = random_spherical_direction(&mut rng);
        let r = Ray::new(*speaker, dir);

        let hits = forward_ray_trace(&r, &geometry, 0, max_bounces, 0., max_distance);

        microphone_hits.extend(
            hits.iter()
                .filter(|(i, _, _)| *i == microphone_index)
                .map(|(_, d, b)| (*d, *b)),
        );
    }

    let inv_speed_of_sound = 1.0 / speed_of_sound;
    let sample_timings: Vec<usize> = microphone_hits
        .iter()
        .map(|(d, _)| (sample_rate * d * inv_speed_of_sound).round() as usize)
        .collect();

    // microphones measure sound pressure, which decays linearly with distance
    let pressures: Vec<f32> = microphone_hits
        .iter()
        .map(|(d, b)| base_impulse / d * (1. - decay).powi(*b as i32))
        .collect();

    let max_timing: usize = sample_timings.iter().cloned().fold(0, usize::max) + 1;
    let mut kernel = vec![0.; max_timing];
    sample_timings
        .iter()
        .zip(pressures)
        .for_each(|(t, e)| kernel[*t] += e);

    let max_k = kernel.iter().cloned().fold(1., f32::max);
    let kernel = kernel.iter().map(|k| k / max_k).collect();
    kernel
}

#[cfg(test)]
mod tests {
    use parry3d::{
        bounding_volume::Aabb,
        math::Point,
        na::Vector3,
        query::{Ray, RayCast},
    };

    use crate::raytracing::forward_ray_trace;

    #[test]
    fn test_forward_ray_trace() {
        let ray = Ray::new(Point::new(5.0, 5.0, 5.0), Vector3::new(0.0, 0.21, 1.0));
        let room = Aabb::new(Point::new(0.0, 0.0, 0.0), Point::new(10.0, 10.0, 10.0));
        let microphone = Aabb::new(Point::new(4.1, 4.1, 9.0), Point::new(5.9, 5.9, 9.1));

        let mut geometry: Vec<&dyn RayCast> = Vec::new();
        geometry.push(&room);
        geometry.push(&microphone);

        let hits = forward_ray_trace(&ray, &geometry, 0, 50, 0.0, 1000.0);
        let microphone_hits = hits
            .iter()
            .filter(|(h, _, _)| *h == geometry.len() - 1)
            .count();

        assert!(hits.len() == 50);
        assert!(microphone_hits == 9);
    }
}
