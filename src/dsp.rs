use sdl2::audio::{AudioCVT, AudioSpec, AudioSpecWAV};
use std::f32::consts::PI;
use std::mem;
use std::path::Path;

pub trait Producer: Send {
    fn write_samples(&mut self, buffer: &mut [f32]);
}

pub struct Sine {
    freq_over_sample_freq: f32,
    offset: f32,
}

impl Sine {
    pub fn new(sample_freq: u32, freq: f32) -> Sine {
        Sine {
            freq_over_sample_freq: freq / (sample_freq as f32),
            offset: 0f32,
        }
    }
}

impl Producer for Sine {
    fn write_samples(&mut self, buffer: &mut [f32]) {
        for i in 0..buffer.len() {
            let t = self.offset + (i as f32);
            buffer[i] = f32::sin(2f32 * PI * self.freq_over_sample_freq * t);
        }
        self.offset += buffer.len() as f32;
    }
}

pub struct StereoSplit<T: Producer> {
    source: T,
}

impl<T: Producer> StereoSplit<T> {
    pub fn new(source: T) -> StereoSplit<T> {
        StereoSplit { source: source }
    }
}

impl<T: Producer> Producer for StereoSplit<T> {
    fn write_samples(&mut self, buffer: &mut [f32]) {
        let mut half_vec = vec![0f32; buffer.len() / 2];
        self.source.write_samples(&mut half_vec);

        for i in 0..buffer.len() {
            buffer[i] = half_vec[i / 2];
        }
    }
}

pub struct WAVProducer {
    data: Vec<f32>,
    pos: usize,
}

impl WAVProducer {
    pub fn new(file_path: &Path, spec: &AudioSpec) -> WAVProducer {
        let wav = AudioSpecWAV::load_wav(file_path).expect("Could not load test WAV file");

        let cvt = AudioCVT::new(
            wav.format,
            wav.channels,
            wav.freq,
            spec.format,
            spec.channels,
            spec.freq,
        ).expect("Could not convert WAV file");

        let data_bytes: Vec<u8> = cvt.convert(wav.buffer().to_vec());

        WAVProducer {
            data: as_floats(data_bytes.as_slice()),
            pos: 0,
        }
    }
}

fn as_floats(v: &[u8]) -> Vec<f32> {
    v.chunks(4)
        .map(|s: &[u8]| unsafe { mem::transmute([s[0], s[1], s[2], s[3]]) })
        .collect()
}

impl Producer for WAVProducer {
    fn write_samples(&mut self, buffer: &mut [f32]) {
        for dst in buffer.iter_mut() {
            *dst = *self.data.get(self.pos).unwrap_or(&0f32);
            self.pos += 1;
        }
    }
}
