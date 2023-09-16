//! Wgpu renderer implemented based on https://sotrh.github.io/learn-wgpu/
mod buffers;
mod state;

use state::State;

use log::debug;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[macro_use]
extern crate static_assertions;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-renderer")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    debug!("Succesfully configured window.");

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id,
            ref event,
        } if window_id == state.window().id() => {
            if !state.input(event) {
                window_event_handler(event, &mut state, control_flow)
            }
        }
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => log::error!("{e:?}"),
            }
        }
        Event::MainEventsCleared => state.window().request_redraw(),
        _ => {}
    });
}

fn window_event_handler(event: &WindowEvent, state: &mut State, control_flow: &mut ControlFlow) {
    use WindowEvent as WE;
    match event {
        WE::CloseRequested => *control_flow = ControlFlow::Exit,
        WE::Resized(physical_size) => state.resize(*physical_size),
        WE::ScaleFactorChanged { new_inner_size, .. } => state.resize(**new_inner_size),
        WE::KeyboardInput { input, .. } => input_handler(input, state, control_flow),
        _ => {}
    };
}

fn input_handler(input: &KeyboardInput, state: &mut State, control_flow: &mut ControlFlow) {
    use KeyboardInput as Input;
    match input {
        Input {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::Escape),
            ..
        } => *control_flow = ControlFlow::Exit,
        Input {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::Q),
            ..
        } => state.next_pipeline(),
        Input {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::Space),
            ..
        } => state.next_buffer(),
        _ => {}
    }
}
