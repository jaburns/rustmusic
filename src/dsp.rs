use sdl2::audio::{AudioCVT, AudioCallback, AudioDevice, AudioSpecDesired, AudioSpecWAV};
use sdl2::{AudioSubsystem};
use std::path::Path;
use std::sync::mpsc::{Receiver};
use std::mem;

pub struct Sound {
    rx: Receiver<bool>,
    data: Vec<f32>,
    volume: f32,
    pos: usize,
}

impl AudioCallback for Sound {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        if self.rx.try_recv().is_ok() {
            self.pos = 0;
        }

        for dst in out.iter_mut() {
            *dst = *self.data.get(self.pos).unwrap_or(&0f32) * self.volume;
            self.pos += 1;
        }
    }
}

fn as_floats(v: &[u8]) -> Vec<f32> {
    v.chunks(4)
        .map(|s: &[u8]| unsafe { mem::transmute([s[0], s[1], s[2], s[3]]) })
        .collect()
}

pub fn create_device(audio: AudioSubsystem, rx: Receiver<bool>) -> AudioDevice<Sound> {
    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(2),
        samples: Some(256),
    };

    let device = audio
        .open_playback(None, &desired_spec, |spec| {
            let wav = AudioSpecWAV::load_wav(Path::new("./kick.wav"))
                .expect("Could not load test WAV file");

            let cvt = AudioCVT::new(
                wav.format,
                wav.channels,
                wav.freq,
                spec.format,
                spec.channels,
                spec.freq,
            ).expect("Could not convert WAV file");

            let data_bytes: Vec<u8> = cvt.convert(wav.buffer().to_vec());

            Sound {
                rx: rx,
                data: as_floats(data_bytes.as_slice()),
                volume: 1.00,
                pos: 1_000_000,
            }
        })
        .unwrap();

    device.resume();

    device
}