use sdl2::audio::{AudioCVT, AudioCallback, AudioDevice, AudioSpecDesired, AudioSpecWAV};
use sdl2::{AudioSubsystem};
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver};
use std::mem;
use std::f32::consts::{PI};

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

trait Producer : Send {
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

struct StereoSplit<T: Producer> { 
    source: T
}

impl<T: Producer> StereoSplit<T> {
    pub fn new(source: T) -> StereoSplit<T> {
        StereoSplit {
            source: source
        }
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

pub struct Device {
    receiver: Receiver<Box<Producer>>,
    producers: Vec<Box<Producer>>,
}

impl Device {
    fn new(receiver: Receiver<Box<Producer>>) -> Device {
        Device {
            receiver: receiver,
            producers: Vec::new(),
        }
    }

    fn receive_producers(&mut self) {
        loop {
            match self.receiver.try_recv() {
                Ok(p) => self.producers.push(p),
                Err(_) => break,
            };
        }
    }
}

impl AudioCallback for Device {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        self.receive_producers();

        if self.producers.len() > 0 {
            self.producers[0].write_samples(out);
        }
    }
}

fn as_floats(v: &[u8]) -> Vec<f32> {
    v.chunks(4)
        .map(|s: &[u8]| unsafe { mem::transmute([s[0], s[1], s[2], s[3]]) })
        .collect()
}

pub fn create_device(audio: AudioSubsystem, rx: Receiver<bool>) -> AudioDevice<Device> {
    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(2),
        samples: Some(256),
    };

    let device = audio
        .open_playback(None, &desired_spec, |spec| {
            /*
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
            */

            let (producer_tx, producer_rx) = mpsc::channel::<Box<Producer>>();
            let result = Device::new(producer_rx);
            producer_tx.send(Box::new(StereoSplit::new(Sine::new(44_100, 440f32)))).unwrap();
            result

        //  Sound {
        //      rx: rx,
        //      data: as_floats(data_bytes.as_slice()),
        //      volume: 1.00,
        //      pos: 1_000_000,
        //  }
        })
        .unwrap();

    device.resume();

    device
}