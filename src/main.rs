extern crate gl;
extern crate imgui;
extern crate imgui_opengl_renderer;
extern crate imgui_sdl2;
extern crate midir;
extern crate sdl2;

use imgui::*;
use midir::{Ignore, MidiInput, MidiInputConnection};
use sdl2::audio::{AudioCVT, AudioCallback, AudioDevice, AudioSpecDesired, AudioSpecWAV};
use sdl2::{AudioSubsystem, EventPump, VideoSubsystem};
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

struct Sound {
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
        .map(|s: &[u8]| unsafe { std::mem::transmute([s[0], s[1], s[2], s[3]]) })
        .collect()
}

fn midi_input_attach_and_listen(tx: Sender<bool>) -> Option<MidiInputConnection<()>> {
    let mut midi_in = MidiInput::new("midir test input").unwrap();

    midi_in.ignore(Ignore::None);
    for i in 0..midi_in.port_count() {
        println!("{}: {}", i, midi_in.port_name(i).unwrap());
    }

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

fn create_audio_device(audio: AudioSubsystem, rx: Receiver<bool>) -> AudioDevice<Sound> {
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

fn start_gui_loop<F>(video: VideoSubsystem, event_pump: &mut EventPump, draw_ui: F)
where
    F: Fn(&Ui),
{
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

    loop {
        use sdl2::event::Event;

        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            if let Event::Quit { .. } = event {
                return;
            }
        }

        let ui = imgui_sdl2.frame(&window, &mut imgui, &event_pump);
        draw_ui(&ui);

        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        renderer.render(ui);
        window.gl_swap_window();

        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let audio = sdl_context.audio().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let (audio_tx, audio_rx) = mpsc::channel::<bool>();

    let _midi = midi_input_attach_and_listen(audio_tx.clone());
    let _audio_device = create_audio_device(audio, audio_rx);

    start_gui_loop(video, &mut event_pump, |ui| {
        ui.window(im_str!("Hello world"))
            .size((300.0, 100.0), ImGuiCond::FirstUseEver)
            .build(|| {
                if ui.button(im_str!("Hello world!"), (0f32, 0f32)) {
                    audio_tx.send(true).unwrap();
                }
            });
    });
}
