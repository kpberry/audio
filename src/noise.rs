use rand::Rng;

fn white_noise(n_samples: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..n_samples).map(|_| (rng.gen::<f32>() - 0.5) * 2.0).collect()
}

#[cfg(test)]
mod tests {
    use std::{fs::File, path::Path};

    use crate::audio::Audio;

    use super::white_noise;

    #[test]
    fn test_noise() {
        let mut file = File::open(Path::new("data/3.wav")).unwrap();
        let mut out_file = File::create("data/3_test.wav").unwrap();
        let mut audio = Audio::from_wav(&mut file).unwrap();
        let noise = white_noise(audio.samples[0].len());
        for i in 0..noise.len() {
            audio.samples[0][i] = audio.samples[0][i] + noise[i] * 0.003;
            audio.samples[1][i] = audio.samples[1][i] + noise[i] * 0.003;
        }
        audio.to_wav(&mut out_file).unwrap();
    }
}