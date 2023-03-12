fn median(samples: &[f32]) -> f32 {
    let mut samples: Vec<f32> = samples.iter().cloned().collect();
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    samples[samples.len() / 2]
}

pub fn median_filter(samples: &[f32], filter_length: usize) -> Vec<f32> {
    (0..samples.len() - filter_length + 1)
        .map(|i| median(&samples[i..i + filter_length]))
        .collect()
}

fn mean(samples: &[f32]) -> f32 {
    samples.iter().sum::<f32>() / samples.len() as f32
}

pub fn mean_filter(samples: &[f32], filter_length: usize) -> Vec<f32> {
    let mut total: f32 = samples.iter().take(filter_length).sum();
    let float_filter_length = filter_length as f32;
    (filter_length..samples.len()).map(|i| {
        total += samples[i] - samples[i - filter_length];
        total / float_filter_length
    }).collect()
}


#[cfg(test)]
mod tests {
    use super::median_filter;
    use super::mean_filter;

    #[test]
    fn test_median_filter() {
        let samples = vec![3., 2., 4., 5., 1., 2., 3., 4., 5., 6., 3., 2., 1.];
        let expected = vec![3., 4., 4., 2., 2., 3., 4., 5., 5., 3., 2.];
        let computed = median_filter(&samples, 3);
        assert!(computed == expected);
    }

    #[test]
    fn test_mean_filter() {
        let samples = vec![3., 2., 4., 5., 1., 2., 3., 4., 5., 6., 3., 2., 1.];
        let expected = vec![3.6666667, 3.3333333, 2.6666667, 2.0, 3.0, 4.0, 5.0, 4.6666665, 3.6666667, 2.0];
        let computed = mean_filter(&samples, 3);
        assert!(computed == expected);
    }
}
