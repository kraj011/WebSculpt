mod state;
mod vertex;
mod texture;
mod camera;
mod instance;
mod model;
mod resources;

use state::State;
use winit::{
    event::*, event_loop::EventLoop, keyboard::{KeyCode, PhysicalKey}, window::WindowBuilder
};

use cgmath::prelude::*;

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn run() {
    // Initialize logging
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Unable to init logger");
        } else {
            env_logger::init();
        }
    }
    


    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window).await;

    let _loop = event_loop.run(move |event, control_flow| match event {
        Event::WindowEvent { window_id, ref event } if window_id == state.window.id() => if !state.input(event) {
            match event {
                WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                    event: KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                    ..
                } => control_flow.exit(),
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                },
                WindowEvent::RedrawRequested => {
                    state.window().request_redraw();

                    //TODO: Check if surface is configured here?
                    
                    state.update();
                    match state.render() {
                        Ok(_) => {},
                        Err(
                            wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated
                        ) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("Out of memory :(");
                            control_flow.exit();
                        },

                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("Surface timed out?");
                        }
                    }
                },
                _ => {}
            }
        },
        _ => {}
    });

    #[cfg(target_arch = "wasm32")]
    {
        use winit::dpi::PhysicalSize;
        let _ = window.request_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(());
            })
            .expect("Unable to append canvas to doc body :(");
    }
}
