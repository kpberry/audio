use std::io::{Error, ErrorKind, Read, Seek, Write};

use wav::{BitDepth, Header};

pub struct Audio {
    pub samples: Vec<Vec<f32>>,
    pub header: Header,
    pub bit_depth: u8,
}

impl Audio {
    pub fn apply(&mut self, f: fn(&[f32]) -> Vec<f32>) {
        self.samples = self.samples.iter().map(|s| f(s)).collect();
    }

    pub fn from_wav<R: Read + Seek>(mut stream: &mut R) -> Result<Audio, std::io::Error> {
        let (header, data) = wav::read(&mut stream)?;

        let (bit_depth, raw_samples): (u8, Result<Vec<f32>, std::io::Error>) = match data {
            wav::BitDepth::Eight(samples) => (
                8,
                Ok(samples
                    .iter()
                    .cloned()
                    .map(|s| s as f32 / 128.0 - 0.5)
                    .collect()),
            ),
            wav::BitDepth::Sixteen(samples) => (
                16,
                Ok(samples
                    .iter()
                    .cloned()
                    .map(|s| s as f32 / 32767.0)
                    .collect()),
            ),
            wav::BitDepth::TwentyFour(samples) => (
                24,
                Ok(samples
                    .iter()
                    .cloned()
                    // we have to divide by an extra 128 because the wav crate reads bytes in the wrong order
                    // TODO submit a pull request to fix this
                    .map(|s| (s as f64 / 8388607.0 / 128.0) as f32)
                    .collect()),
            ),
            wav::BitDepth::ThirtyTwoFloat(samples) => (32, Ok(samples.iter().cloned().collect())),
            wav::BitDepth::Empty => (
                0,
                Err(Error::new(
                    ErrorKind::InvalidData,
                    "Bit depth was empty; could not load samples.",
                )),
            ),
        };
        let raw_samples = raw_samples?;

        // cache friendly file loading; ensures that we don't have to load multiple channel sample vectors in cache at a time
        let n_channels = header.channel_count as usize;
        let mut samples: Vec<Vec<f32>> = Vec::with_capacity(n_channels);
        for channel in 0..n_channels {
            let mut channel_samples: Vec<f32> = Vec::new();
            for &s in raw_samples.iter().skip(channel).step_by(n_channels) {
                channel_samples.push(s);
            }
            samples.push(channel_samples);
        }

        let n_samples = samples[0].len();
        for channel_samples in &samples {
            if channel_samples.len() != n_samples {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "All audio channels should have the same number of samples.",
                ));
            }
        }

        Ok(Audio {
            samples,
            header,
            bit_depth,
        })
    }

    pub fn to_wav<W: Write + Seek>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        let n_samples = self.samples[0].len();
        let n_channels = self.header.channel_count as usize;
        let mut raw_samples: Vec<f32> = vec![0.; n_samples * self.samples.len()];
        for (channel, channel_samples) in self.samples.iter().enumerate() {
            (channel..raw_samples.len())
                .step_by(n_channels)
                .zip(channel_samples)
                .for_each(|(i, &s)| {
                    raw_samples[i] = s;
                });
        }

        let track = match self.bit_depth {
            8 => Ok(BitDepth::Eight(
                raw_samples
                    .iter()
                    .cloned()
                    .map(|s| ((s + 0.5) * 128.) as u8)
                    .collect(),
            )),
            16 => Ok(BitDepth::Sixteen(
                raw_samples
                    .iter()
                    .cloned()
                    .map(|s| (s * 32767.0) as i16)
                    .collect(),
            )),
            24 => Ok(BitDepth::TwentyFour(
                raw_samples
                    .iter()
                    .cloned()
                    // we have to multiply by an extra 128 because the wav crate reads bytes in the wrong order
                    .map(|s| (s * 8388607. * 128.0) as i32)
                    .collect(),
            )),
            32 => Ok(BitDepth::ThirtyTwoFloat(
                raw_samples.iter().cloned().map(|s| s as f32).collect(),
            )),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid bit depth; could not flatten samples.",
            )),
        }?;

        wav::write(self.header, &track, writer)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, path::Path};

    use super::Audio;

    #[test]
    fn test_audio_from_wav() {
        let mut file = File::open(Path::new("data/3.wav")).unwrap();
        Audio::from_wav(&mut file).unwrap();
    }

    #[test]
    fn test_audio_to_wav() {
        let mut file = File::open(Path::new("data/3.wav")).unwrap();
        let mut out_file = File::create("data/3_test.wav").unwrap();
        let audio = Audio::from_wav(&mut file).unwrap();
        audio.to_wav(&mut out_file).unwrap();
    }
}
