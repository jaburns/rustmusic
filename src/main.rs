extern crate gl;
extern crate imgui;
extern crate imgui_opengl_renderer;
extern crate imgui_sdl2;
extern crate midir;
extern crate sdl2;

use midir::{Ignore, MidiInput};
use sdl2::audio::{AudioCVT, AudioCallback, AudioSpecDesired, AudioSpecWAV};
use std::path::{Path};
use std::time::Duration;

struct Sound {
    data: Vec<f32>,
    volume: f32,
    pos: usize,
}

impl AudioCallback for Sound {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for dst in out.iter_mut() {
            *dst = *self.data.get(self.pos).unwrap_or(&0f32) * self.volume;
            self.pos += 1;
        }
    }
}

fn as_floats(v: &[u8]) -> Vec<f32> {
    v.chunks(4)
        .map(|s: &[u8]| unsafe { std::mem::transmute([s[0], s[1], s[2], s[3]]) })
        .collect()
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 0);

    let window = video
        .window("Rust Music", 1024, 768)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .unwrap();

    let _gl_context = window
        .gl_create_context()
        .expect("Couldn't create GL context");
    gl::load_with(|s| video.gl_get_proc_address(s) as _);

    let mut imgui = imgui::ImGui::init();
    imgui.set_ini_filename(None);

    let mut imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui);

    let renderer =
        imgui_opengl_renderer::Renderer::new(&mut imgui, |s| video.gl_get_proc_address(s) as _);

    let mut event_pump = sdl_context.event_pump().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(2),
        samples: Some(256),
    };

    let device = audio_subsystem
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
                data: as_floats(data_bytes.as_slice()),
                volume: 1.00,
                pos: 0,
            }
        })
        .unwrap();

    device.resume();

    let mut midi_in = MidiInput::new("midir test input").unwrap();
    midi_in.ignore(Ignore::None);
    for i in 0..midi_in.port_count() {
        println!("{}: {}", i, midi_in.port_name(i).unwrap());
    }

    'running: loop {
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;

        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            if imgui_sdl2.ignore_event(&event) {
                continue;
            }

            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        let ui = imgui_sdl2.frame(&window, &mut imgui, &event_pump);
        ui.show_test_window(&mut true);

        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        renderer.render(ui);

        window.gl_swap_window();

        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}