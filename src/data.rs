//
// Pointing Simulator
// Copyright (c) 2023 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use cgmath::{Deg, Point3, Rad};
use crate::gui::CameraView;
use glium::program;
use std::{cell::RefCell, rc::Rc};

#[derive(Copy, Clone)]
pub struct Vertex2 {
    pub position: [f32; 2]
}
glium::implement_vertex!(Vertex2, position);

#[derive(Copy, Clone)]
pub struct Vertex3 {
    pub position: [f32; 3]
}
glium::implement_vertex!(Vertex3, position);

#[derive(Clone)]
pub struct MeshBuffers {
    pub vertices: Rc<glium::VertexBuffer<Vertex3>>,
    pub indices: Rc<glium::IndexBuffer<u32>>,
}

pub struct OpenGlObjects {
    pub sky_mesh: MeshBuffers,
    pub sky_mesh_prog: Rc<glium::Program>,
    pub texture_copy_single: Rc<glium::Program>,
    pub texture_copy_multi: Rc<glium::Program>,
    pub unit_quad: Rc<glium::VertexBuffer<Vertex2>>,
}

pub struct ProgramData {
    pub camera_view: CameraView,
    gl_objects: OpenGlObjects
}

impl ProgramData {
    pub fn new(renderer: &Rc<RefCell<imgui_glium_renderer::Renderer>>, display: &glium::Display) -> ProgramData {
        let sky_mesh_prog = Rc::new(program!(display,
            330 => {
                vertex: include_str!("resources/shaders/3d_view.vert"),
                fragment: include_str!("resources/shaders/solid_color.frag"),
            }
        ).unwrap());

        let texture_copy_single = Rc::new(program!(display,
            330 => {
                vertex: include_str!("resources/shaders/pass-through.vert"),
                fragment: include_str!("resources/shaders/texturing.frag"),
            }
        ).unwrap());

        let texture_copy_multi = Rc::new(program!(display,
            330 => {
                vertex: include_str!("resources/shaders/pass-through.vert"),
                fragment: include_str!("resources/shaders/texturing_multi-sample.frag"),
            }
        ).unwrap());

        let unit_quad_data = [
            Vertex2{ position: [-1.0, -1.0] },
            Vertex2{ position: [ 1.0, -1.0] },
            Vertex2{ position: [ 1.0,  1.0] },
            Vertex2{ position: [-1.0,  1.0] }
        ];
        let unit_quad = Rc::new(glium::VertexBuffer::new(display, &unit_quad_data).unwrap());

        let gl_objects = OpenGlObjects{
            sky_mesh: create_sky_mesh(Deg(10.0), 10, display),
            sky_mesh_prog,
            texture_copy_single,
            texture_copy_multi,
            unit_quad
        };

        ProgramData{
            camera_view: CameraView::new(&gl_objects, renderer, display),
            gl_objects
        }
    }
}

fn create_sky_mesh(
    step: cgmath::Deg<f32>,
    num_substeps: usize,
    display: &glium::Display
) -> MeshBuffers {
    let mut vertex_data: Vec<Vertex3> = vec![];
    let mut index_data: Vec<u32> = vec![];

    let mut longitude = cgmath::Deg(-180.0);
    while longitude <= cgmath::Deg(180.0) {
        let mut latitude = cgmath::Deg(-90.0);
        let mut parallel_starts = true;
        while latitude <= cgmath::Deg(90.0) {
            vertex_data.push(Vertex3{ position: *to_xyz_unit(latitude, longitude).as_ref() });
            if !parallel_starts {
                index_data.push((vertex_data.len() - 2) as u32);
                index_data.push((vertex_data.len() - 1) as u32);
            }
            latitude += step / num_substeps as f32;
            parallel_starts = false;
        }

        longitude += step;
    }

    let mut latitude = cgmath::Deg(-90.0);
    while latitude <= cgmath::Deg(90.0) {
        let mut longitude = cgmath::Deg(-180.0);
        let mut meridian_starts = true;
        while longitude <= cgmath::Deg(180.0) {
            vertex_data.push(Vertex3{ position: *to_xyz_unit(latitude, longitude).as_ref() });
            if !meridian_starts {
                index_data.push((vertex_data.len() - 2) as u32);
                index_data.push((vertex_data.len() - 1) as u32);
            }
            longitude += step / num_substeps as f32;
            meridian_starts = false;
        }

        latitude += step;
    }

    let vertices = Rc::new(glium::VertexBuffer::new(display, &vertex_data).unwrap());
    let indices = Rc::new(glium::IndexBuffer::new(display, glium::index::PrimitiveType::LinesList, &index_data).unwrap());

    MeshBuffers{ vertices, indices }
}

/// Coordinates in Cartesian frame with lat. 0°, lon. 0° being (1, 0, 0) and the North Pole at (0, 0, 1).
fn to_xyz_unit(lat: Deg<f32>, lon: Deg<f32>) -> Point3<f32> {
    Point3{
        x: Rad::from(lon).0.cos() * Rad::from(lat).0.cos(),
        y: Rad::from(lon).0.sin() * Rad::from(lat).0.cos(),
        z: Rad::from(lat).0.sin()
    }
}
