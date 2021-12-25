use std::f64::consts::PI;
use num::complex::Complex;
use num::Zero;

pub fn _fft(xs: &Vec<Complex<f64>>, sign: f64) -> Vec<Complex<f64>> {
    let n = xs.len();
    assert!(n.is_power_of_two());

    if n == 1 {
        vec![xs[0]]
    } else {
        let xs_even = _fft(&xs.iter().cloned().step_by(2).collect(), sign);
        let xs_odd = _fft(&xs.iter().cloned().skip(1).step_by(2).collect(), sign);

        let mut result = vec![Complex::new(0.0, 0.0); n];
        let n_inv = 1.0 / n as f64;
        let half_n = n >> 1;

        for k in 0..half_n {
            let p = xs_even[k];
            let q = Complex::new(0.0, sign * 2.0 * PI * n_inv * (k as f64)).exp() * xs_odd[k];
            result[k] = p + q;
            result[k + half_n] = p - q;
        }
        result
    }
}

pub fn fft(xs: &Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    _fft(xs, -1.0)
}

pub fn ifft(xs: &Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    let n = xs.len() as f64;
    _fft(xs, 1.0).iter().map(|x| x / n).collect()
}

pub fn fft_convolve(xs: &Vec<Complex<f64>>, ys: &Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    let len = (xs.len() + ys.len() - 1);
    let xs = xs.iter().cloned()
        .chain(std::iter::repeat(Complex::zero()))
        .take(len.next_power_of_two())
        .collect();
    let ys = ys.iter().cloned()
        .chain(std::iter::repeat(Complex::zero()))
        .take(len.next_power_of_two())
        .collect();
    let fxs = fft(&xs);
    let fys = fft(&ys);
    let product = fxs.iter().zip(fys).map(|(x, y)| x * y).collect();
    ifft(&product).into_iter().take(len).collect()
}

pub fn rfft_convolve(xs: &Vec<f64>, ys: &Vec<f64>) -> Vec<f64> {
    fft_convolve(
        &xs.iter().map(|&x| Complex::new(x, 0.0)).collect(),
        &ys.iter().map(|&y| Complex::new(y, 0.0)).collect(),
    ).iter().map(|p| p.re).collect()
}

#[test]
fn test_rft() {
    let fftd = fft(&vec![1., 2., 3., 4.].into_iter().map(|r| Complex::new(r, 0.)).collect());
    let expected = vec![
        Complex::new(10., 0.), Complex::new(-2., 2.),
        Complex::new(-2., 0.), Complex::new(-2., -2.)
    ];
    assert_eq!(expected.len(), fftd.len());
    assert!(fftd.iter().zip(expected).all(|(a, b)| (a - b).norm() < 1e-7));
}

#[test]
fn test_rfft_convolve() {
    let convolved = rfft_convolve(&vec![1., 2., 3., 4.], &vec![1., 2., 3., 4., 5., 6., 7., 8.]);
    let expected = vec![1., 4., 10., 20., 30., 40., 50., 60., 61., 52., 32.];
    assert_eq!(expected.len(), convolved.len());
    assert!(convolved.iter().zip(expected).all(|(a, b)| (a - b).abs() < 1e-7));
}