use std::cell::RefCell;
use std::sync::Arc;

use rustfft::FftPlanner;
use vst::buffer::AudioBuffer;
use vst::plugin::{HostCallback, Info, Plugin, PluginParameters};
use vst::util::AtomicFloat;

use crate::convolution::{rfft_convolve};
use crate::geometry::{make_box, P, Q, Visible};
use crate::raytracing::profile_room;

struct RayVerb {
    params: Arc<RayVerbParameters>,
    kernel: Arc<Option<Vec<f64>>>,
    // this is a pretty terrible hack to help with storing the buffers we need
    signal_buffers: Arc<Vec<Vec<f64>>>,
    planner: Arc<RefCell<FftPlanner<f64>>>,
}

impl Default for RayVerb {
    fn default() -> Self {
        RayVerb {
            params: Arc::new(RayVerbParameters::default()),
            kernel: Arc::new(None),
            signal_buffers: Arc::new(Vec::new()),
            planner: Arc::new(RefCell::new(FftPlanner::new())),
        }
    }
}

struct RayVerbParameters {
    room_width: AtomicFloat,
    room_height: AtomicFloat,
    room_depth: AtomicFloat,
    speaker_x: AtomicFloat,
    speaker_y: AtomicFloat,
    speaker_z: AtomicFloat,
    microphone_width: AtomicFloat,
    microphone_height: AtomicFloat,
    microphone_x: AtomicFloat,
    microphone_y: AtomicFloat,
    microphone_z: AtomicFloat,
    samples: AtomicFloat,
    max_bounces: AtomicFloat,
    max_delay: AtomicFloat,
    speed_of_sound: AtomicFloat,
    decay: AtomicFloat,
}

impl Default for RayVerbParameters {
    fn default() -> Self {
        RayVerbParameters {
            room_width: AtomicFloat::new(100.0),
            room_height: AtomicFloat::new(100.0),
            room_depth: AtomicFloat::new(100.0),
            speaker_x: AtomicFloat::new(50.0),
            speaker_y: AtomicFloat::new(5.0),
            speaker_z: AtomicFloat::new(5.0),
            microphone_width: AtomicFloat::new(5.0),
            microphone_height: AtomicFloat::new(5.0),
            microphone_x: AtomicFloat::new(50.0),
            microphone_y: AtomicFloat::new(5.0),
            microphone_z: AtomicFloat::new(95.0),
            samples: AtomicFloat::new(100000.0),
            max_bounces: AtomicFloat::new(10.0),
            max_delay: AtomicFloat::new(30.0),
            speed_of_sound: AtomicFloat::new(343.0),
            decay: AtomicFloat::new(0.05),
        }
    }
}

impl PluginParameters for RayVerbParameters {
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.room_width.get() / 200.0,
            1 => self.room_height.get() / 200.0,
            2 => self.room_depth.get() / 200.0,
            3 => self.speaker_x.get() / 200.0,
            4 => self.speaker_y.get() / 200.0,
            5 => self.speaker_z.get() / 200.0,
            6 => self.microphone_width.get() / 10.0,
            7 => self.microphone_height.get() / 10.0,
            8 => self.microphone_x.get() / 200.0,
            9 => self.microphone_y.get() / 200.0,
            10 => self.microphone_z.get() / 200.0,
            11 => self.samples.get() / 1000000.0,
            12 => self.max_bounces.get() / 30.0,
            13 => self.max_delay.get() / 150.0,
            14 => self.speed_of_sound.get() / 10000.0,
            15 => self.decay.get() / 1.0,
            _ => 0.0,
        }
    }

    fn set_parameter(&self, index: i32, val: f32) {
        match index {
            0 => self.room_width.set(val * 200.0),
            1 => self.room_height.set(val * 200.0),
            2 => self.room_depth.set(val * 200.0),
            3 => self.speaker_x.set(val * 200.0),
            4 => self.speaker_y.set(val * 200.0),
            5 => self.speaker_z.set(val * 200.0),
            6 => self.microphone_width.set(val * 10.0),
            7 => self.microphone_height.set(val * 10.0),
            8 => self.microphone_x.set(val * 200.0),
            9 => self.microphone_y.set(val * 200.0),
            10 => self.microphone_z.set(val * 200.0),
            11 => self.samples.set((val * 1000000.0).round()),
            12 => self.max_bounces.set((val * 30.0).round()),
            13 => self.max_delay.set(val * 150.0),
            14 => self.speed_of_sound.set(val * 10000.0),
            15 => self.decay.set(val * 1.0),
            _ => (),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{:.2}", self.room_width.get()),
            1 => format!("{:.2}", self.room_height.get()),
            2 => format!("{:.2}", self.room_depth.get()),
            3 => format!("{:.2}", self.speaker_x.get()),
            4 => format!("{:.2}", self.speaker_y.get()),
            5 => format!("{:.2}", self.speaker_z.get()),
            6 => format!("{:.2}", self.microphone_width.get()),
            7 => format!("{:.2}", self.microphone_height.get()),
            8 => format!("{:.2}", self.microphone_x.get()),
            9 => format!("{:.2}", self.microphone_y.get()),
            10 => format!("{:.2}", self.microphone_z.get()),
            11 => format!("{:.0}", self.samples.get()),
            12 => format!("{:.0}", self.max_bounces.get()),
            13 => format!("{:.2}", self.max_delay.get()),
            14 => format!("{:.2}", self.speed_of_sound.get()),
            15 => format!("{:.2}", self.decay.get()),
            _ => "".to_string(),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Room Width",
            1 => "Room Height",
            2 => "Room Depth",
            3 => "Speaker X",
            4 => "Speaker Y",
            5 => "Speaker Z",
            6 => "Microphone Width",
            7 => "Microphone Height",
            8 => "Microphone X",
            9 => "Microphone Y",
            10 => "Microphone Z",
            11 => "Samples",
            12 => "Max Bounces",
            13 => "Max Delay",
            14 => "Speed of Sound",
            15 => "Decay",
            _ => "",
        }.to_string()
    }
}

impl Plugin for RayVerb {
    fn get_info(&self) -> Info {
        Info {
            name: "RayVerb".to_string(),
            unique_id: 1729,
            inputs: 2,
            outputs: 2,
            parameters: 16,
            ..Default::default()
        }
    }

    fn new(_host: HostCallback) -> Self {
        RayVerb::default()
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        if self.kernel.is_none() {
            let room_width = self.params.room_width.get() as f64;
            let room_height = self.params.room_height.get() as f64;
            let room_depth = self.params.room_depth.get() as f64;
            let speaker_x = self.params.speaker_x.get() as f64;
            let speaker_y = self.params.speaker_y.get() as f64;
            let speaker_z = self.params.speaker_z.get() as f64;
            let mx = self.params.microphone_x.get() as f64;
            let my = self.params.microphone_y.get() as f64;
            let mz = self.params.microphone_z.get() as f64;
            let mw = self.params.microphone_width.get() as f64;
            let mh = self.params.microphone_height.get() as f64;
            let samples = self.params.samples.get() as usize;
            let max_bounces = self.params.max_bounces.get() as usize;
            let max_delay = self.params.max_delay.get() as f64;
            let speed_of_sound = self.params.speed_of_sound.get() as f64;
            let decay = self.params.decay.get() as f64;

            let room = make_box(
                room_width,
                room_height,
                room_depth,
                P {
                    x: 0.,
                    y: 0.,
                    z: 0.,
                },
            );
            let room_ref = room.iter().map(|r| r as &dyn Visible).collect();

            let speaker = P {
                x: speaker_x,
                y: speaker_y,
                z: speaker_z,
            };

            let microphone = Q::cw(
                P { x: mx - mw / 2.0, y: my + mh / 2.0, z: mz },
                P { x: mx + mw / 2.0, y: my + mh / 2.0, z: mz },
                P { x: mx + mw / 2.0, y: my - mh / 2.0, z: mz },
                P { x: mx - mw / 2.0, y: my - mh / 2.0, z: mz },
            );

            let kernel = profile_room(
                &room_ref,
                &speaker,
                &microphone,
                samples,
                max_bounces,
                max_delay,
                speed_of_sound,
                decay,
                1.,
                44100.,
            );

            let kernel_len = kernel.len();
            self.kernel = Arc::new(Some(kernel));
            self.signal_buffers = Arc::new((0..buffer.input_count()).map(|_| vec![0.0; kernel_len]).collect());
        }

        let kernel = (*self.kernel).as_ref().unwrap();
        let mut next_signal_buffers: Vec<Vec<f64>> = Vec::new();
        let mut planner = self.planner.borrow_mut();

        for ((input_buffer, output_buffer), signal_buffer) in buffer.zip().zip(&*self.signal_buffers) {
            let start_index = signal_buffer.len().saturating_sub(kernel.len());
            let signal = signal_buffer.iter()
                .skip(start_index)
                .cloned()
                .chain(input_buffer.iter().map(|&f| f as f64))
                .collect();
            let response = rfft_convolve(&signal_buffer, kernel, &mut planner);
            for (&r, o) in response.iter().skip(kernel.len()).zip(output_buffer.iter_mut()) {
                *o = r as f32;
            }
            next_signal_buffers.push(signal);
        }

        self.signal_buffers = Arc::new(next_signal_buffers);
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }

    fn start_process(&mut self) {
        self.kernel = Arc::new(None);
        self.signal_buffers = Arc::new(Vec::new());
    }

    fn stop_process(&mut self) {
        self.kernel = Arc::new(None);
        self.signal_buffers = Arc::new(Vec::new());
    }
}

plugin_main!(RayVerb);
