use core::ReceivesAudioSpec;
use dsp::{Producer, WAVProducer, Sine, StereoSplit};
use midi::{MidiMessage, MidiMessageKind};
use sdl2::audio::{AudioCallback, AudioSpec};
use std::path::Path;
use std::sync::mpsc::Receiver;

pub struct GUIEvent {}

pub struct Device {
    audio_spec: Option<AudioSpec>,
    midi_receiver: Receiver<MidiMessage>,
    gui_receiver: Receiver<GUIEvent>,
    producers: Vec<Box<Producer>>,
}

impl Device {
    pub fn new(midi_receiver: Receiver<MidiMessage>, gui_receiver: Receiver<GUIEvent>) -> Device {
        Device {
            midi_receiver: midi_receiver,
            gui_receiver: gui_receiver,
            producers: Vec::new(),
            audio_spec: None,
        }
    }
}

impl ReceivesAudioSpec for Device {
    fn receive_spec(&mut self, spec: AudioSpec) {
        self.audio_spec = Some(spec);
    }
}

impl AudioCallback for Device {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        match self.gui_receiver.try_recv() {
            Ok(_) => {
                self.producers.push(Box::new(WAVProducer::new(
                    Path::new("./kick.wav"),
                    &self.audio_spec.unwrap(),
                )));
            }
            Err(_) => {}
        }

        match self.midi_receiver.try_recv() {
            Ok(MidiMessage { kind, key, .. }) => {
                if kind == MidiMessageKind::KeyPress {
                    self.producers.push(Box::new(StereoSplit::new(Sine::new(44_100, 440f32))));
                }
            }
            Err(_) => {}
        }

        let mut mixing_buffer = vec![0f32; out.len()];

        for i in 0..out.len() {
            out[i] = 0f32;
        }

        for producer in self.producers.iter_mut() {
            producer.write_samples(mixing_buffer.as_mut_slice());

            for i in 0..out.len() {
                out[i] += mixing_buffer[i];
            }
        }
    }
}