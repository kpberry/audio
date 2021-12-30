use std::f64::consts::PI;

use num::integer::Roots;
use num::Zero;
use rand::seq::index::sample;
use rustfft::{Direction, Fft, FftPlanner, num_complex::Complex};

pub fn fft_convolve(signal: &Vec<Complex<f64>>, kernel: &Vec<Complex<f64>>,
                    planner: &mut FftPlanner<f64>) -> Vec<Complex<f64>> {
    // the lengths here are very important; don't change them unless you know what you're doing
    let len = signal.len() + kernel.len() - 1;
    let buf_len = len.next_power_of_two();
    let fft = planner.plan_fft_forward(buf_len);
    let ifft = planner.plan_fft_inverse(buf_len);

    let mut f_signal: Vec<Complex<f64>> = signal.iter().cloned()
        .chain(std::iter::repeat(Complex::zero()))
        .take(buf_len)
        .collect();
    fft.process(&mut f_signal);

    let mut f_kernel: Vec<Complex<f64>> = kernel.iter().cloned()
        .chain(std::iter::repeat(Complex::zero()))
        .take(buf_len)
        .collect();
    fft.process(&mut f_kernel);

    // we have to manually normalize before doing the inverse fft
    let norm = 1. / buf_len as f64;
    let mut normed_product: Vec<Complex<f64>> = f_signal.iter().zip(f_kernel)
        .map(|(x, y)| x * y * norm)
        .collect();
    ifft.process(&mut normed_product);
    normed_product.into_iter().take(len).collect()
}

pub fn rfft_convolve(signal: &Vec<f64>, kernel: &Vec<f64>,
                     planner: &mut FftPlanner<f64>) -> Vec<f64> {
    fft_convolve(
        &signal.iter().map(|&x| Complex::new(x, 0.0)).collect(),
        &kernel.iter().map(|&y| Complex::new(y, 0.0)).collect(),
        planner,
    ).iter().map(|p| p.re).collect()
}

#[test]
fn test_rfft_convolve() {
    let convolved = rfft_convolve(
        &vec![
            1., 2., 3., 4., 5., 6., 7., 8., 1., 2., 3., 4., 5., 6., 7., 8.,
            1., 2., 3., 4., 5., 6., 7., 8., 1., 2., 3., 4., 5., 6., 7., 8.,
            1., 2., 3., 4., 5., 6., 7., 8., 1., 2., 3., 4., 5., 6., 7., 8.,
            1., 2., 3., 4., 5., 6., 7., 8., 1., 2., 3., 4., 5., 6., 7., 8.,
            1., 2., 3., 4., 5., 6., 7., 8., 1., 2., 3., 4., 5., 6., 7.,
        ],
        &vec![
            1., 2., 3., 4., 5., 6., 7., 8., 1., 2., 3., 4., 5., 6., 7., 8.,
        ],
        &mut FftPlanner::new(),
    );
    let expected = vec![
        1.00, 4.00, 10.00, 20.00, 35.00, 56.00, 84.00, 120.00, 149.00, 172.00, 190.00,
        204.00, 215.00, 224.00, 232.00, 240.00, 296.00, 336.00, 360.00, 368.00, 360.00,
        336.00, 296.00, 240.00, 296.00, 336.00, 360.00, 368.00, 360.00, 336.00, 296.00,
        240.00, 296.00, 336.00, 360.00, 368.00, 360.00, 336.00, 296.00, 240.00, 296.00,
        336.00, 360.00, 368.00, 360.00, 336.00, 296.00, 240.00, 296.00, 336.00, 360.00,
        368.00, 360.00, 336.00, 296.00, 240.00, 296.00, 336.00, 360.00, 368.00, 360.00,
        336.00, 296.00, 240.00, 296.00, 336.00, 360.00, 368.00, 360.00, 336.00, 296.00,
        240.00, 296.00, 336.00, 360.00, 368.00, 360.00, 336.00, 296.00, 232.00, 279.00,
        308.00, 318.00, 308.00, 277.00, 224.00, 148.00, 112.00, 131.00, 140.00, 138.00,
        124.00, 97.00, 56.00,
    ];
    assert_eq!(expected.len(), convolved.len());
    assert!(convolved.iter().zip(expected).all(|(a, b)| (a - b).abs() < 1e-7));
}

fn rfft_convolve_real_time(signal: &Vec<f64>, kernel: &Vec<f64>, sample_size: usize,
                           planner: &mut FftPlanner<f64>) -> Vec<f64> {
    let mut input_buffers = Vec::new();
    let mut output_buffers = Vec::new();
    for i in (0..signal.len()).step_by(sample_size) {
        let input_buffer: Vec<f64> = signal.iter().skip(i).take(sample_size).cloned().collect();
        let buf_len = input_buffer.len();
        input_buffers.push(input_buffer);

        let output_buffer: Vec<f64> = vec![0.0; buf_len];
        output_buffers.push(output_buffer);
    }

    let mut signal_buffer = vec![0.0; kernel.len()];
    for (input_buffer, output_buffer) in input_buffers.iter().zip(output_buffers.iter_mut()) {
        let start_index = signal_buffer.len().saturating_sub(kernel.len());
        signal_buffer = signal_buffer.iter()
            .skip(start_index)
            .cloned()
            .chain(input_buffer.iter().map(|&f| f))
            .collect();
        let response = rfft_convolve(&signal_buffer, kernel, planner);
        for (&r, o) in response.iter().skip(kernel.len()).zip(output_buffer.iter_mut()) {
            *o = r;
        }
    }

    output_buffers.iter().cloned().flatten().collect()
}

#[test]
fn test_rfft_convolve_real_time() {
    let convolved = rfft_convolve_real_time(
        &vec![
            1., 2., 3., 4., 5., 6., 7., 8., 1., 2., 3., 4., 5., 6., 7., 8.,
            1., 2., 3., 4., 5., 6., 7., 8., 1., 2., 3., 4., 5., 6., 7., 8.,
            1., 2., 3., 4., 5., 6., 7., 8., 1., 2., 3., 4., 5., 6., 7., 8.,
            1., 2., 3., 4., 5., 6., 7., 8., 1., 2., 3., 4., 5., 6., 7., 8.,
            1., 2., 3., 4., 5., 6., 7., 8., 1., 2., 3., 4., 5., 6., 7.,
        ],
        &vec![
            1., 2., 3., 4., 5., 6., 7., 8., 1., 2., 3., 4., 5., 6., 7., 8.,
        ],
        5,
        &mut FftPlanner::new(),
    );
    let expected = vec![
        1.00, 4.00, 10.00, 20.00, 35.00, 56.00, 84.00, 120.00, 149.00, 172.00, 190.00,
        204.00, 215.00, 224.00, 232.00, 240.00, 296.00, 336.00, 360.00, 368.00, 360.00,
        336.00, 296.00, 240.00, 296.00, 336.00, 360.00, 368.00, 360.00, 336.00, 296.00,
        240.00, 296.00, 336.00, 360.00, 368.00, 360.00, 336.00, 296.00, 240.00, 296.00,
        336.00, 360.00, 368.00, 360.00, 336.00, 296.00, 240.00, 296.00, 336.00, 360.00,
        368.00, 360.00, 336.00, 296.00, 240.00, 296.00, 336.00, 360.00, 368.00, 360.00,
        336.00, 296.00, 240.00, 296.00, 336.00, 360.00, 368.00, 360.00, 336.00, 296.00,
        240.00, 296.00, 336.00, 360.00, 368.00, 360.00, 336.00, 296.00, 232.00, 279.00,
        308.00, 318.00, 308.00, 277.00, 224.00, 148.00, 112.00, 131.00, 140.00, 138.00,
        124.00, 97.00, 56.00,
    ];
    assert!(convolved.len() >= expected.len() - 16);
    assert!(convolved.iter().zip(expected).all(|(a, b)| (a - b).abs() < 1e-7));
}