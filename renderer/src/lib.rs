//! Wgpu renderer implemented based on https://sotrh.github.io/learn-wgpu/
mod assets;
mod camera;
mod instance;
mod model;
mod state;
mod texture;

use state::State;

use log::debug;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");

            // // Canvas jpeg load texture
            // let texture = web_sys::window()
            //     .and_then(|win| win.document())
            //     .and_then(|doc| {
            //         let canvas = doc.get_element_by_id("wasm-texture").unwrap();
            //         let canvas: web_sys::HtmlCanvasElement = canvas
            //             .dyn_into::<web_sys::HtmlCanvasElement>()
            //             .map_err(|_| ())
            //             .unwrap();
            //         let context = canvas
            //             .get_context("2d")
            //             .unwrap()
            //             .unwrap()
            //             .dyn_into::<web_sys::CanvasRenderingContext2d>()
            //             .unwrap();
            //         Some(
            //             context
            //                 .get_image_data(
            //                     0.0,
            //                     0.0,
            //                     canvas
            //                         .get_attribute("width")
            //                         .unwrap()
            //                         .parse()
            //                         .expect("Couldn't parse canvas width."),
            //                     canvas
            //                         .get_attribute("height")
            //                         .unwrap()
            //                         .parse()
            //                         .expect("Couldn't parse canvas height,"),
            //                 )
            //                 .expect("Couldn't get image data.."),
            //         )
            //     })
            //     .expect("Couldn't load texture from canvas.");
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
        } if window_id == state.window().id() => state.input(event, control_flow),
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
