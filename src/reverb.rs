use std::{fs::File, io::Write, path::Path};

use crate::raytracing::{make_box, profile_room, Visible, P, S};

pub fn kernel_reverb(samples: &Vec<f64>, kernel: &Vec<f64>) -> Vec<f64> {
    let mut result = vec![0.; samples.len()];
    for (i, s) in samples.iter().enumerate() {
        println!("{}", i);
        let end = (i + kernel.len()).min(samples.len());
        kernel[..end - i]
            .iter()
            .enumerate()
            .for_each(|(j, k)| result[i + j] += k * s);
    }
    result
}

// c [-2.0, -3.5, -4.0, -3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0]
// w [-2.0, -3.5, -4.0, -3.0, -2.0, -4.5, -4.0, 1.0, 2.0, -4.5]
pub fn _kernel_reverb(samples: &Vec<f64>, kernel: &Vec<f64>) -> Vec<f64> {
    let mut result = vec![0.; samples.len()];

    let buf_len = kernel.len();
    let mut buf = vec![0.; buf_len];
    let kernel: Vec<f64> = kernel.iter().cloned().rev().collect();
    let mut remaining = samples.len();
    for i in (0..samples.len()).step_by(buf_len) {
        println!("{}", i);
        for t in 0..(buf_len.min(remaining)) {
            println!("{}", t);
            buf[t] = samples[i + t];
            result[i + t] = buf
                .iter()
                .zip(
                    // rotated kernel; note that we don't need to slice the left half because zip
                    // ends as soon as the shorter sequence (buf) ends
                    kernel[(buf_len - t - 1)..].iter().chain(kernel.iter()),
                )
                .fold(0., |x, (s, k)| x + s * k);
        }
        // avoids both conditionals and underflow when nearing the end of the buffer
        remaining = remaining.saturating_sub(buf_len);
    }
    result
}