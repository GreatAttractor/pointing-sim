//
// Pointing Simulator
// Copyright (c) 2023-2024 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

mod camera_view;
mod draw_buffer;

use crate::{data, runner, workers::MountState};
use glium::glutin::surface::WindowSurface;
use pointing_utils::uom;
use std::{cell::RefCell, rc::Rc};
use uom::si::angle;

pub use camera_view::CameraView;

/// Zoom factor per one step of mouse wheel.
const MOUSE_WHEEL_ZOOM_FACTOR: f32 = 1.1;

#[derive(Default)]
pub struct GuiState {
    hidpi_factor: f64,
    // pub mouse_drag_origin: [f32; 2],
    // pub message_box: Option<MessageBox>,
    pub font_size: f32,
    pub provisional_font_size: Option<f32>
}

impl GuiState {
    pub fn new(hidpi_factor: f64, font_size: f32) -> GuiState {
        GuiState{
            hidpi_factor,
            font_size,
            ..Default::default()
        }
    }

    pub fn hidpi_factor(&self) -> f64 { self.hidpi_factor }
}

pub struct AdjustedImageSize {
    pub logical_size: [f32; 2],
    pub physical_size: [u32; 2]
}

pub fn handle_gui(
    program_data: &mut data::ProgramData,
    ui: &imgui::Ui,
    renderer: &Rc<RefCell<imgui_glium_renderer::Renderer>>,
    display: &glium::Display<WindowSurface>
) -> Option<runner::FontSizeRequest> {
    unsafe { imgui::sys::igDockSpaceOverViewport(
        imgui::sys::igGetMainViewport(),
        imgui::sys::ImGuiDockNodeFlags_PassthruCentralNode as i32,
        std::ptr::null()
    ); }

    handle_camera_view(
        &mut program_data.camera_view.borrow_mut(),
        ui,
        &mut program_data.gui_state,
        &program_data.mount.get()
    );

    None
}

fn handle_camera_view(
    camera_view: &mut CameraView,
    ui: &imgui::Ui,
    gui_state: &mut GuiState,
    mount_state: &MountState
) {
    ui.window(&format!("Camera view"))
        .size([640.0, 640.0], imgui::Condition::FirstUseEver)
        .build(|| {
            let hidpi_f = gui_state.hidpi_factor as f32;

            let adjusted = adjust_pos_for_exact_hidpi_scaling(ui, 0.0, hidpi_f);

            camera_view.update_size(
                adjusted.physical_size[0],
                adjusted.physical_size[1]
            );

            camera_view.set_mount_state(mount_state);

            let image_start_pos = ui.cursor_pos();
            imgui::Image::new(camera_view.draw_buf_id(), adjusted.logical_size).build(ui);

            if ui.is_item_hovered() {
                let wheel = ui.io().mouse_wheel;
                if wheel != 0.0 {
                    let zoom_factor = MOUSE_WHEEL_ZOOM_FACTOR.powf(wheel);
                    camera_view.zoom_by(zoom_factor);
                }
            }

            ui.set_cursor_pos(image_start_pos);
            let _disabled = ui.begin_disabled(true);
            let _token1 = ui.push_style_color(imgui::StyleColor::Text, [0.0, 0.0, 0.0, 1.0]);
            let _token2 = ui.push_style_color(imgui::StyleColor::Button, [1.0, 1.0, 1.0, 0.8]);
            let a1deg = mount_state.axis1_pos.get::<angle::degree>();
            ui.small_button(&format!(
                "az. {:.1}°, alt. {:.1}°\nFOVy {:.02}°",
                if a1deg >= 0.0 && a1deg <= 180.0 { a1deg } else { 360.0 + a1deg },
                mount_state.axis2_pos.get::<angle::degree>(),
                camera_view.field_of_view_y().0
            ));
        });
}

/// Adjusts cursor screen position and returns size to be used for an `imgui::Image` (meant to fill the remaining window
/// space) to ensure exact 1:1 pixel rendering when high-DPI scaling is enabled.
pub fn adjust_pos_for_exact_hidpi_scaling(
    ui: &imgui::Ui,
    vertical_space_after: f32,
    hidpi_factor: f32
) -> AdjustedImageSize {
    let scr_pos = ui.cursor_screen_pos();

    let adjusted_pos_x = if (scr_pos[0] * hidpi_factor).fract() != 0.0 {
        (scr_pos[0] * hidpi_factor).trunc() / hidpi_factor
    } else {
        scr_pos[0]
    };

    let adjusted_pos_y = if (scr_pos[1] * hidpi_factor).fract() != 0.0 {
        (scr_pos[1] * hidpi_factor).trunc() / hidpi_factor
    } else {
        scr_pos[1]
    };

    ui.set_cursor_screen_pos([adjusted_pos_x, adjusted_pos_y]);

    let mut size = ui.content_region_avail();
    size[1] -= vertical_space_after;

    let mut adjusted_size_x = size[0].trunc();
    if (adjusted_size_x * hidpi_factor).fract() != 0.0 {
        adjusted_size_x = (adjusted_size_x * hidpi_factor).trunc() / hidpi_factor;
    }

    let mut adjusted_size_y = size[1].trunc();
    if (adjusted_size_y * hidpi_factor).fract() != 0.0 {
        adjusted_size_y = (adjusted_size_y * hidpi_factor).trunc() / hidpi_factor;
    }

    let physical_size = [
        (adjusted_size_x * hidpi_factor).trunc() as u32,
        (adjusted_size_y * hidpi_factor).trunc() as u32
    ];

    AdjustedImageSize{
        logical_size: [adjusted_size_x, adjusted_size_y],
        physical_size
    }
}
