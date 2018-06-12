use core::ReceivesAudioSpec;
use sdl2::audio::{AudioCallback, AudioSpec};
use std::sync::mpsc::Receiver;
use midi::MidiMessage;
use dsp::Producer;

pub struct Device {
    audio_spec: Option<AudioSpec>,
    midi_receiver: Receiver<MidiMessage>,
    producers: Vec<Box<Producer>>,
}

impl Device {
    pub fn new(midi_receiver: Receiver<MidiMessage>) -> Device {
        Device {
            midi_receiver: midi_receiver,
            producers: Vec::new(),
            audio_spec: None,
        }
    }
}

impl AudioCallback for Device {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        if self.producers.len() > 0 {
            self.producers[0].write_samples(out);
        }
    }
}

impl ReceivesAudioSpec for Device {
    fn receive_spec(&mut self, spec: AudioSpec) {
        self.audio_spec = Some(spec);
    }
}