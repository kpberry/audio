use std::f64::consts::PI;

use num::integer::Roots;
use num::Zero;
use rustfft::{Direction, Fft, FftPlanner, num_complex::Complex};

pub fn fft_convolve(signal: &Vec<Complex<f64>>, kernel: &Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    // the lengths here are very important; don't change them unless you know what you're doing
    let len = signal.len() + kernel.len() - 1;
    let buf_len = len.next_power_of_two();
    let mut planner = FftPlanner::new();
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

pub fn rfft_convolve(signal: &Vec<f64>, kernel: &Vec<f64>) -> Vec<f64> {
    fft_convolve(
        &signal.iter().map(|&x| Complex::new(x, 0.0)).collect(),
        &kernel.iter().map(|&y| Complex::new(y, 0.0)).collect(),
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