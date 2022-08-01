use std::collections::HashMap;

pub trait IntervalTuningSystem {
    fn octave_interval(&self) -> i32;
    fn octave(&self, freq: f64) -> i32;
    fn position(&self, freq: f64) -> f64;
    fn freq(&self, position: f64, octave: i32) -> f64;
    fn add_interval(&self, freq: f64, interval: f64) -> f64;
    fn sub_interval(&self, freq: f64, interval: f64) -> f64 {
        self.add_interval(freq, -interval)
    }
}

#[derive(Debug)]
pub struct EqualTemperament {
    num_tones: i32,
    base_freq: f64,
    base_octave: i32,
}

impl IntervalTuningSystem for EqualTemperament {
    fn octave_interval(&self) -> i32 {
        self.num_tones
    }

    fn octave(&self, freq: f64) -> i32 {
        self.base_octave + (freq / self.base_freq).log2().floor() as i32
    }

    fn position(&self, freq: f64) -> f64 {
        (freq / self.base_freq).log2() * self.num_tones as f64
    }

    fn freq(&self, position: f64, octave: i32) -> f64 {
        self.base_freq
            * ((octave - self.base_octave) as f64 + position / self.num_tones as f64).exp2()
    }

    fn add_interval(&self, freq: f64, interval: f64) -> f64 {
        //        The commented code here is equivalent to the actual code, but slower.
        //        self.freq(self.position(freq) + interval, self.octave(freq))
        freq * (interval / self.num_tones as f64).exp2()
    }
}

pub fn a440() -> EqualTemperament {
    EqualTemperament {
        num_tones: 12,
        base_freq: 440.0 * (-0.75 as f64).exp2(),
        base_octave: 4,
    }
}

pub trait NamingSystem {
    fn standardize_name(&self, name: &str) -> Option<String>;
    fn name_to_position(&self, name: &str) -> Option<f64>;
}

#[derive(Debug)]
pub struct NoteNames {
    names_to_positions: HashMap<String, f64>,
}

impl NamingSystem for NoteNames {
    fn standardize_name(&self, name: &str) -> Option<String> {
        let name = name.to_uppercase();
        match name.len() {
            1 => Some(name),
            2 => {
                let chars: Vec<char> = name.chars().collect();
                let (note, accidental) = (chars[0], chars[1]);
                let accidental = match accidental {
                    'B' | '♭' => Some('♭'),
                    '#' | '♯' => Some('♯'),
                    '♮' => Some('♮'),
                    _ => None,
                };
                accidental.map(|accidental| format!("{}{}", note, accidental))
            },
            _ => None
        }
    }

    fn name_to_position(&self, name: &str) -> Option<f64> {
        self.standardize_name(name).map(|name| self.names_to_positions.get(&name).map(|&position| position))?
    }
}

fn western_naming_system() -> NoteNames {
    let names_to_positions = vec![
        (String::from("B♯"), 0.0),
        (String::from("C"), 0.0),
        (String::from("C♯"), 1.0),
        (String::from("D♭"), 1.0),
        (String::from("D"), 2.0),
        (String::from("D♯"), 3.0),
        (String::from("E♭"), 3.0),
        (String::from("E"), 4.0),
        (String::from("F♭"), 4.0),
        (String::from("E♯"), 5.0),
        (String::from("F"), 5.0),
        (String::from("F♯"), 6.0),
        (String::from("G♭"), 6.0),
        (String::from("G"), 7.0),
        (String::from("G♯"), 8.0),
        (String::from("A♭"), 8.0),
        (String::from("A"), 9.0),
        (String::from("A♯"), 10.0),
        (String::from("B♭"), 10.0),
        (String::from("B"), 11.0),
        (String::from("C♭"), 11.0),
    ]
    .into_iter()
    .collect();
    NoteNames {
        names_to_positions,
    }
}

pub fn demo() {
    let ts = a440();
    let ns = western_naming_system();

    let note =
        |name: &str, octave: i32| -> f64 { ts.freq(ns.name_to_position(name).unwrap(), octave) };
    println!("{}", note("A", 0));
}
