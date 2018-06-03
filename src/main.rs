extern crate gl;
extern crate imgui;
extern crate imgui_opengl_renderer;
extern crate imgui_sdl2;
extern crate midir;
extern crate sdl2;

mod dsp;
mod midi;
mod gui;

use imgui::*;
use std::sync::mpsc;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let audio = sdl_context.audio().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let (audio_tx, audio_rx) = mpsc::channel::<bool>();

    let _midi = midi::listen_to_input(audio_tx.clone());
    let _audio_device = dsp::create_device(audio, audio_rx);

    gui::run(video, &mut event_pump, |ui| {
        ui.window(im_str!("Hello world"))
            .size((300.0, 100.0), ImGuiCond::FirstUseEver)
            .build(|| {
                if ui.button(im_str!("Hello world!"), (0f32, 0f32)) {
                    audio_tx.send(true).unwrap();
                }

                PlotLines::new(ui, im_str!("hello"), &[-10f32,1f32,20f32,1f32])
                    .graph_size((0f32, 100f32))
                    .build();
            });
    });
}
