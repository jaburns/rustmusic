use std;
use gl;
use imgui;
use imgui_sdl2;
use imgui_opengl_renderer;
use imgui::*;
use sdl2;
use sdl2::{EventPump, VideoSubsystem};

pub fn run<F>(video: VideoSubsystem, event_pump: &mut EventPump, draw_ui: F)
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