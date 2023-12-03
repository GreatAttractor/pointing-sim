//
// Pointing Simulator
// Copyright (c) 2023 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use cgmath::{Deg, EuclideanSpace, Matrix4, Point3, SquareMatrix, Vector3};
use crate::{data, gui::draw_buffer::{DrawBuffer, Sampling}};
use glium::{Surface, uniform};
use std::{cell::RefCell, rc::Rc};

pub struct CameraView {
    dir: Vector3<f32>,
    up: Vector3<f32>,
    field_of_view_y: Deg<f32>,
    draw_buf: DrawBuffer,
    gl_projection: Matrix4<f32>,
    gl_view: Matrix4<f32>,
    sky_mesh: data::MeshBuffers,
    sky_mesh_prog: Rc<glium::Program>,
}

impl CameraView {
    pub fn new(
        gl_objects: &data::OpenGlObjects,
        renderer: &Rc<RefCell<imgui_glium_renderer::Renderer>>,
        display: &glium::Display
    ) -> CameraView {
        let field_of_view_y = Deg(40.0);
        let dir = Vector3{ x: 1.0, y: 0.0, z: 0.5 };
        let up = Vector3{ x: 0.0, y: 0.0, z: 1.0 };

        CameraView{
            dir,
            up,
            field_of_view_y,
            draw_buf: DrawBuffer::new(
                Sampling::Multi,
                &gl_objects.texture_copy_single,
                &gl_objects.texture_copy_multi,
                &gl_objects.unit_quad,
                display,
                &renderer
            ),
            gl_projection: cgmath::perspective(field_of_view_y, 1.0, 0.1, 5.0),
            gl_view: Matrix4::look_to_rh(Point3::origin(), dir, up),
            sky_mesh: gl_objects.sky_mesh.clone(),
            sky_mesh_prog: gl_objects.sky_mesh_prog.clone()
        }
    }

    pub fn update_size(&mut self, width: u32, height: u32) {
        if self.draw_buf.update_size(width, height) {
            self.gl_projection = cgmath::perspective(self.field_of_view_y, width as f32 / height as f32, 0.1, 5.0);
            self.render()
        }
    }

    fn render(&self) {
        let mut target = self.draw_buf.frame_buf();
        target.clear_color_and_depth((0.5, 0.5, 1.0, 1.0), 1.0);

        let uniforms = uniform! {
            model: Into::<[[f32; 4]; 4]>::into(Matrix4::<f32>::identity()),
            view: Into::<[[f32; 4]; 4]>::into(self.gl_view),
            projection: Into::<[[f32; 4]; 4]>::into(self.gl_projection),
            draw_color: [0.0f32, 0.0f32, 0.0f32, 1.0f32]
        };

        target.draw(
            &*self.sky_mesh.vertices,
            &*self.sky_mesh.indices,
            &self.sky_mesh_prog,
            &uniforms,
            &glium::DrawParameters{
                depth: glium::Depth{
                    test: glium::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                ..Default::default()
            }
        ).unwrap();

        self.draw_buf.update_storage_buf();
    }

    pub fn draw_buf_id(&self) -> imgui::TextureId { self.draw_buf.id() }
}
