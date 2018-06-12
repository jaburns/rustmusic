extern crate gl;
extern crate imgui;
extern crate imgui_opengl_renderer;
extern crate imgui_sdl2;
extern crate midir;
extern crate sdl2;

mod audio;
mod core;
mod dsp;
mod midi;

use imgui::*;
use mpsc::channel;
use std::sync::mpsc;

fn main() {
    let (midi_tx, midi_rx) = channel::<midi::MidiMessage>();
    let (gui_tx, gui_rx) = channel::<audio::GUIEvent>();

    let _midi = midi::listen_to_input(midi_tx.clone());
    let dsp_device = audio::Device::new(midi_rx, gui_rx);

    core::run(dsp_device, 0f32, |ui, state| {
        ui.window(im_str!("Hello world"))
            .size((300.0, 100.0), ImGuiCond::FirstUseEver)
            .build(|| {
                if ui.button(im_str!("Hello world!"), (0f32, 0f32)) {
                    gui_tx.send(audio::GUIEvent {}).unwrap();
                }

                SliderFloat::new(ui, im_str!("Slider"), state, 0f32, 10f32).build();

                PlotLines::new(ui, im_str!("hello"), &[-10f32, 1f32, 20f32, 1f32])
                    .graph_size((0f32, 100f32))
                    .build();
            });
    });
}
