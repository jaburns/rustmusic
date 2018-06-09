use midir::{Ignore, MidiInput, MidiInputConnection};
use std::sync::mpsc::{Sender};

pub fn listen_to_input(tx: Sender<bool>) -> Option<MidiInputConnection<()>> {
    let mut midi_in = MidiInput::new("midir test input").unwrap();

    midi_in.ignore(Ignore::None);
    for i in 0..midi_in.port_count() {
        println!("{}: {}", i, midi_in.port_name(i).unwrap());
    }

    return None;

    if midi_in.port_count() > 0 {
        Some(
            midi_in
                .connect(
                    0,
                    "midir-forward",
                    move |_, message, _| {
                        if message[0] == 152 && message[2] != 0 {
                            tx.send(true).unwrap();
                        }
                    },
                    (),
                )
                .unwrap(),
        )
    } else {
        None
    }
}