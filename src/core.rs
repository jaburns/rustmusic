use gl;
use imgui;
use imgui::*;
use imgui_opengl_renderer;
use imgui_sdl2;
use sdl2;
use sdl2::audio::{AudioCallback, AudioSpec, AudioSpecDesired};
use std;

pub const SAMPLE_FREQUENCY: i32 = 44_100;
pub const SAMPLE_COUNT: usize = 256;
pub const CHANNEL_COUNT: usize = 2;
pub const SAMPLE_BUFFER_SIZE: usize = SAMPLE_COUNT * CHANNEL_COUNT;

pub static ZERO_BUFFER: [f32; SAMPLE_BUFFER_SIZE] = [0f32; SAMPLE_BUFFER_SIZE];

pub trait ReceivesAudioSpec {
    fn receive_spec(&mut self, spec: AudioSpec);
}

pub fn run<F, S, CB>(mut audio_cb: CB, mut gui_state: S, gui_cb: F)
where
    F: Fn(&Ui, &mut S),
    CB: AudioCallback + ReceivesAudioSpec,
{
    let sdl_context = sdl2::init().unwrap();

    // Init audio

    let audio = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLE_FREQUENCY),
        channels: Some(2),
        samples: Some(256),
    };

    let device = audio
        .open_playback(None, &desired_spec, |spec| {
            audio_cb.receive_spec(spec);
            audio_cb
        })
        .unwrap();

    device.resume();

    // Init video

    let video = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

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

    // Run GUI thread

    loop {
        use sdl2::event::Event;

        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            if let Event::Quit { .. } = event {
                return;
            }
        }

        let ui = imgui_sdl2.frame(&window, &mut imgui, &event_pump);
        gui_cb(&ui, &mut gui_state);

        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        renderer.render(ui);
        window.gl_swap_window();

        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}
