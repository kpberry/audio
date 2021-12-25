use std::f64::consts::PI;

use num::integer::Roots;
use num::Zero;
use rustfft::{Direction, Fft, FftPlanner, num_complex::Complex};

pub fn fft_convolve(xs: &Vec<Complex<f64>>, ys: &Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    // the lengths here are very important; don't change them unless you know what you're doing
    let len = (xs.len() + ys.len() - 1);
    let buf_len = len.next_power_of_two();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(buf_len);
    let ifft = planner.plan_fft_inverse(buf_len);

    let mut fxs: Vec<Complex<f64>> = xs.iter().cloned()
        .chain(std::iter::repeat(Complex::zero()))
        .take(buf_len)
        .collect();
    fft.process(&mut fxs);

    let mut fys: Vec<Complex<f64>> = ys.iter().cloned()
        .chain(std::iter::repeat(Complex::zero()))
        .take(buf_len)
        .collect();
    fft.process(&mut fys);

    // we have to manually normalize before doing the inverse fft
    let norm = 1. / buf_len as f64;
    let mut normed_product: Vec<Complex<f64>> = fxs.iter().zip(fys)
        .map(|(x, y)| x * y * norm)
        .collect();
    ifft.process(&mut normed_product);
    normed_product.into_iter().take(len).collect()
}

pub fn rfft_convolve(xs: &Vec<f64>, ys: &Vec<f64>) -> Vec<f64> {
    fft_convolve(
        &xs.iter().map(|&x| Complex::new(x, 0.0)).collect(),
        &ys.iter().map(|&y| Complex::new(y, 0.0)).collect(),
    ).iter().map(|p| p.re).collect()
}

#[test]
fn test_rfft_convolve() {
    let convolved = rfft_convolve(&vec![1., 2., 3., 4.], &vec![1., 2., 3., 4., 5., 6., 7., 8.]);
    let expected = vec![1., 4., 10., 20., 30., 40., 50., 60., 61., 52., 32.];
    assert_eq!(expected.len(), convolved.len());
    assert!(convolved.iter().zip(expected).all(|(a, b)| (a - b).abs() < 1e-7));
}