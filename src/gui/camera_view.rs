//
// Pointing Simulator
// Copyright (c) 2023 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use cgmath::{Basis3, Deg, EuclideanSpace, InnerSpace, Matrix3, Matrix4, Point3, Rotation3, SquareMatrix, Vector3};
use crate::{data, data::{MeshVertex, Vertex3}, gui::draw_buffer::{DrawBuffer, Sampling}};
use glium::{Surface, uniform};
use std::{cell::RefCell, rc::Rc};

pub struct CameraView {
    dir: Vector3<f32>,
    up: Vector3<f32>,
    field_of_view_y: Deg<f32>,
    draw_buf: DrawBuffer,
    gl_view: Matrix4<f32>,
    sky_mesh: data::MeshBuffers<Vertex3>,
    sky_mesh_prog: Rc<glium::Program>,
    target_mesh: data::MeshBuffers<MeshVertex>,
    target_prog: Rc<glium::Program>,
    target_pos: Point3<f32>,
    target_heading: Deg<f32>,
    wh_ratio: f32
}

impl CameraView {
    pub fn new(
        gl_objects: &data::OpenGlObjects,
        renderer: &Rc<RefCell<imgui_glium_renderer::Renderer>>,
        display: &glium::Display
    ) -> CameraView {
        let field_of_view_y = Deg(2.0);
        let target_pos = Point3{ x: 2000.0, y: 0.0, z: 500.0 };
        let dir = target_pos.to_vec();
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
            gl_view: Matrix4::look_to_rh(Point3::origin(), dir, up),
            sky_mesh: gl_objects.sky_mesh.clone(),
            sky_mesh_prog: gl_objects.sky_mesh_prog.clone(),
            target_mesh: gl_objects.target_mesh.clone(),
            target_prog: gl_objects.target_prog.clone(),
            target_pos,
            target_heading: Deg(-45.0),
            wh_ratio: 1.0
        }
    }

    fn gl_projection(&self, near: f32, far: f32) -> Matrix4<f32> {
        cgmath::perspective(self.field_of_view_y, self.wh_ratio, near, far)
    }

    pub fn update_size(&mut self, width: u32, height: u32) {
        if self.draw_buf.update_size(width, height) {
            self.wh_ratio = width as f32 / height as f32;
            self.render()
        }
    }

    fn render(&self) {
        let mut target = self.draw_buf.frame_buf();
        target.clear_color_and_depth((0.5, 0.5, 1.0, 1.0), 1.0);

        let uniforms = uniform! {
            model: Into::<[[f32; 4]; 4]>::into(Matrix4::<f32>::identity()),
            view: Into::<[[f32; 4]; 4]>::into(self.gl_view),
            projection: Into::<[[f32; 4]; 4]>::into(self.gl_projection(0.1, 5.0)),
            draw_color: [0.0f32, 0.0f32, 0.0f32, 1.0f32]
        };
        target.draw(
            &*self.sky_mesh.vertices,
            &*self.sky_mesh.indices,
            &self.sky_mesh_prog,
            &uniforms,
            &glium::DrawParameters{
                depth: glium::Depth{
                    test: glium::DepthTest::Overwrite,
                    write: false,
                    ..Default::default()
                },
                ..Default::default()
            }
        ).unwrap();


        let target_dist = self.target_pos.to_vec().magnitude();
        assert!(target_dist > 500.0);
        let target_model = Matrix4::<f32>::from_translation(self.target_pos.to_vec())
            * Matrix4::from(Matrix3::from(Basis3::from_angle_z(-self.target_heading)));
        let uniforms = uniform! {
            model: Into::<[[f32; 4]; 4]>::into(target_model),
            view: Into::<[[f32; 4]; 4]>::into(self.gl_view),
            projection: Into::<[[f32; 4]; 4]>::into(self.gl_projection(target_dist - 70.0, target_dist + 70.0)),
            draw_color: [0.0f32, 1.0f32, 0.0f32, 1.0f32]
        };
        target.draw(
            &*self.target_mesh.vertices,
            &*self.target_mesh.indices,
            &self.target_prog,
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
