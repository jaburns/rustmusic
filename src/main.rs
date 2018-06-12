extern crate gl;
extern crate imgui;
extern crate imgui_opengl_renderer;
extern crate imgui_sdl2;
extern crate midir;
extern crate sdl2;

mod core;
mod dsp;
mod midi;
mod audio;

use imgui::*;
use std::sync::mpsc;

fn main() {
    let (midi_tx, midi_rx) = mpsc::channel::<midi::MidiMessage>();

    let _midi = midi::listen_to_input(audio_tx.clone());
    let dsp_device = dsp::Device::new();

    core::run(dsp_device, 0f32, |ui, state| {
        ui.window(im_str!("Hello world"))
            .size((300.0, 100.0), ImGuiCond::FirstUseEver)
            .build(|| {
                if ui.button(im_str!("Hello world!"), (0f32, 0f32)) {
                    audio_tx.send(true).unwrap();
                }

                SliderFloat::new(ui, im_str!("Junker"), state, 0f32, 10f32).build();

                PlotLines::new(ui, im_str!("hello"), &[-10f32, 1f32, 20f32, 1f32])
                    .graph_size((0f32, 100f32))
                    .build();
            });
    });
}
