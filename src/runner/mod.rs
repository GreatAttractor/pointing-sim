//
// Pointing Simulator
// Copyright (c) 2023 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use glium::{
    Surface,
    glutin::{
        config::ConfigTemplateBuilder,
        context::{ContextAttributesBuilder, NotCurrentGlContext},
        display::{GetGlDisplay, GlDisplay},
        surface::{SurfaceAttributesBuilder, WindowSurface}
    }
};
use imgui_winit_support::winit::{
    dpi,
    event,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder}
};
use raw_window_handle::HasRawWindowHandle;
use std::{cell::RefCell, num::NonZeroU32, rc::Rc};

mod clipboard_support;

#[derive(Copy, Clone)]
pub struct FontSizeRequest(pub f32);

pub struct Runner {
    event_loop: EventLoop<()>,
    display: glium::Display<WindowSurface>,
    imgui: imgui::Context,
    pub window: Window,
    platform: imgui_winit_support::WinitPlatform,
    renderer: Rc<RefCell<imgui_glium_renderer::Renderer>>
}

fn create_font(physical_font_size: f32) -> imgui::FontSource<'static> {
    imgui::FontSource::TtfData{
        data: include_bytes!(
            "../resources/fonts/DejaVuSans.ttf"
        ),
        size_pixels: physical_font_size,
        config: Some(imgui::FontConfig {
            glyph_ranges: imgui::FontGlyphRanges::from_slice(&[
                0x0020, 0x00FF, // Basic Latin, Latin-1 Supplement
                '▶' as u32, '▶' as u32,
                '■' as u32, '■' as u32,
                '⟳' as u32, '⟳' as u32,
                '⇄' as u32, '⇄' as u32,
                '⚙' as u32, '⚙' as u32,
                0
            ]),
            ..imgui::FontConfig::default()
        }),
    }.into()
}

pub fn create_runner(logical_font_size: f32) -> Runner {
    const INITIAL_WIDTH: u32 = 1024;
    const INITIAL_HEIGHT: u32 = 768;

    let event_loop = EventLoop::new().expect("Failed to create EventLoop");

    let window_builder = WindowBuilder::new()
        .with_title("Pointing Simulator".to_owned())
        .with_inner_size(dpi::LogicalSize::new(INITIAL_WIDTH as f64, INITIAL_HEIGHT as f64));

    let (window, cfg) = glutin_winit::DisplayBuilder::new()
        .with_window_builder(Some(window_builder))
        .build(&event_loop, ConfigTemplateBuilder::new(), |mut configs| {
            configs.next().unwrap()
        })
        .expect("Failed to create OpenGL window");
    let window = window.unwrap();

    let context_attribs = ContextAttributesBuilder::new().build(Some(window.raw_window_handle()));
    let context = unsafe {
        cfg.display()
            .create_context(&cfg, &context_attribs)
            .expect("Failed to create OpenGL context")
    };

    let surface_attribs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        window.raw_window_handle(),
        NonZeroU32::new(INITIAL_WIDTH).unwrap(),
        NonZeroU32::new(INITIAL_HEIGHT).unwrap(),
    );

    let surface = unsafe {
        cfg.display()
            .create_window_surface(&cfg, &surface_attribs)
            .expect("Failed to create OpenGL surface")
    };

    let context = context
        .make_current(&surface)
        .expect("Failed to make OpenGL context current");

    let display = glium::Display::from_context_surface(context, surface)
        .expect("Failed to create glium Display");


    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);

    if let Some(backend) = clipboard_support::init() {
        imgui.set_clipboard_backend(backend);
    } else {
        eprintln!("Failed to initialize clipboard.");
    }

    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
    platform.attach_window(imgui.io_mut(), &window, imgui_winit_support::HiDpiMode::Default);

    let hidpi_factor = platform.hidpi_factor() as f32;
    let font_size = logical_font_size * hidpi_factor;

    imgui.fonts().add_font(&[create_font(font_size)]);

    imgui.io_mut().font_global_scale = 1.0 / hidpi_factor;
    imgui.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;
    imgui.io_mut().config_windows_move_from_title_bar_only = true;

    let renderer = imgui_glium_renderer::Renderer::init(&mut imgui, &display).expect("failed to initialize renderer");

    Runner{
        event_loop,
        display,
        imgui,
        window,
        platform,
        renderer: Rc::new(RefCell::new(renderer))
    }
}

impl Runner {
    pub fn platform(&self) -> &imgui_winit_support::WinitPlatform {
        &self.platform
    }

    pub fn display(&self) -> &glium::Display<WindowSurface> {
        &self.display
    }

    pub fn main_loop<F>(self, mut run_ui: F)
        where F: FnMut(
            &mut bool,
            &mut imgui::Ui,
            &glium::Display<WindowSurface>,
            &Rc<RefCell<imgui_glium_renderer::Renderer>>
        ) -> Option<FontSizeRequest> + 'static
    {
        let Runner {
            event_loop,
            display,
            mut imgui,
            window,
            mut platform,
            renderer,
            ..
        } = self;

        let mut last_frame = std::time::Instant::now();

        event_loop.run(move |event, window_target| match event {
            Event::NewEvents(_) => {
                let now = std::time::Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            },

            Event::AboutToWait => {
                platform
                    .prepare_frame(imgui.io_mut(), &window)
                    .expect("Failed to prepare frame");
                window.request_redraw();
            },

            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let font_size_request;
                {
                    let mut ui = imgui.frame();

                    let mut run = true;
                    font_size_request = run_ui(&mut run, &mut ui, &display, &renderer);
                    if !run {
                        window_target.exit();
                    }

                    let mut target = display.draw();
                    target.clear_color_srgb(0.5, 0.5, 0.5, 1.0);
                    platform.prepare_render(&ui, &window);
                    let draw_data = imgui.render();
                    renderer.borrow_mut()
                        .render(&mut target, draw_data)
                        .expect("rendering failed");
                    target.finish().expect("failed to swap buffers");
                }
                if let Some(fsr) = font_size_request {
                    imgui.fonts().clear();
                    imgui.fonts().add_font(&[create_font(platform.hidpi_factor() as f32 * fsr.0)]);
                    renderer.borrow_mut().reload_font_texture(&mut imgui).unwrap();
                }
            },

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => window_target.exit(),

            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                if new_size.width > 0 && new_size.height > 0 {
                    display.resize((new_size.width, new_size.height));
                }
                platform.handle_event(imgui.io_mut(), &window, &event);
            },

            event => {
                let converted_event = convert_touch_to_mouse(event);

                platform.handle_event(imgui.io_mut(), &window, &converted_event);
            }
        }).expect("EventLoop error");
    }
}

fn convert_touch_to_mouse<'a, T>(event: Event<T>) -> Event<T> {
    match event {
        Event::WindowEvent {
            window_id,
            event: WindowEvent::Touch(touch),
        } => {
            //TODO: do something better here, e.g. remember the last seen mouse device id
            let device_id = touch.device_id.clone();

            match touch.phase {
                event::TouchPhase::Started => event::Event::WindowEvent{
                    window_id: window_id.clone(),
                    event: event::WindowEvent::MouseInput{
                        device_id,
                        state: event::ElementState::Pressed,
                        button: event::MouseButton::Left,
                    },
                },

                event::TouchPhase::Ended => event::Event::WindowEvent{
                    window_id: window_id.clone(),
                    event: event::WindowEvent::MouseInput{
                        device_id,
                        state: event::ElementState::Released,
                        button: event::MouseButton::Left,
                    },
                },

                _ => event
            }
        },

        _ => event
    }
}
