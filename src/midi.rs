use midir::{Ignore, MidiInput, MidiInputConnection};
use std::sync::mpsc::Sender;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MidiMessageKind {
    KeyPress,
    KeyRelease,
}

#[derive(Copy, Clone)]
pub struct MidiMessage {
    pub kind: MidiMessageKind,
    pub key: u8,
    pub velocity: u8,
}

pub fn listen_to_input(sender: Sender<MidiMessage>) -> Option<MidiInputConnection<()>> {
    let mut midi_in = MidiInput::new("midir test input").unwrap();

    midi_in.ignore(Ignore::None);
    for i in 0..midi_in.port_count() {
        println!("{}: {}", i, midi_in.port_name(i).unwrap());
    }

    let on_message = move |_: u64, message: &[u8], _: &mut ()| {
        if message[0] == 152 {
            sender
                .send(MidiMessage {
                    kind: if message[2] == 0 {
                        MidiMessageKind::KeyPress
                    } else {
                        MidiMessageKind::KeyRelease
                    },
                    key: message[1],
                    velocity: message[2],
                })
                .unwrap();
        }
    };

    if midi_in.port_count() > 0 {
        Some(midi_in.connect(0, "midir-forward", on_message, ()).unwrap())
    } else {
        None
    }
}
