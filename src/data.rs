//
// Pointing Simulator
// Copyright (c) 2023-2024 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use cgmath::{Basis3, Deg, EuclideanSpace, InnerSpace, Rad, Rotation, Rotation3};
use crate::{gui::CameraView, workers::Mount, target_interpolator::TargetInterpolator};
use glium::program;
use pointing_utils::{TargetInfoMessage, LatLon, to_global_unit};
use std::{cell::RefCell, error::Error, rc::Rc, sync::Arc};

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

#[derive(Copy, Clone)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3]
}
glium::implement_vertex!(MeshVertex, position, normal);

#[derive(Clone)]
pub struct MeshBuffers<T: Copy> {
    pub vertices: Rc<glium::VertexBuffer<T>>,
    pub indices: Rc<glium::IndexBuffer<u32>>,
}

pub struct OpenGlObjects {
    pub sky_mesh: MeshBuffers<Vertex3>,
    pub sky_mesh_prog: Rc<glium::Program>,
    pub texture_copy_single: Rc<glium::Program>,
    pub texture_copy_multi: Rc<glium::Program>,
    pub unit_quad: Rc<glium::VertexBuffer<Vertex2>>,
    pub target_mesh: MeshBuffers<MeshVertex>,
    pub target_prog: Rc<glium::Program>
}

pub struct ProgramData {
    pub camera_view: Rc<RefCell<CameraView>>,
    gl_objects: OpenGlObjects,
    pub gui_state: crate::gui::GuiState,
    pub target_receiver: crossbeam::channel::Receiver<TargetInfoMessage>,
    pub target_subscribers: subscriber_rs::SubscriberCollection<TargetInfoMessage>,
    pub target_interpolator: Rc<RefCell<TargetInterpolator>>,
    pub mount: Arc<Mount>
}

impl ProgramData {
    pub fn new(
        renderer: &Rc<RefCell<imgui_glium_renderer::Renderer>>,
        display: &glium::Display,
        gui_state: crate::gui::GuiState,
        target_receiver: crossbeam::channel::Receiver<TargetInfoMessage>,
        mount: Arc<Mount>
    ) -> ProgramData {
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

        let target_prog = Rc::new(program!(display,
            330 => {
                vertex: include_str!("resources/shaders/3d_view.vert"),
                fragment: include_str!("resources/shaders/surface.frag"),
            }
        ).unwrap());

        let gl_objects = OpenGlObjects{
            sky_mesh: create_sky_mesh(Deg(10.0), 10, display),
            sky_mesh_prog,
            texture_copy_single,
            texture_copy_multi,
            unit_quad,
            target_mesh: create_target_mesh(display),
            target_prog
        };

        let camera_view = Rc::new(RefCell::new(CameraView::new(&gl_objects, renderer, display)));

        let target_interpolator = Rc::new(RefCell::new(TargetInterpolator::new()));
        target_interpolator.borrow_mut().add_subscriber(Rc::downgrade(&camera_view) as _);

        let mut target_subscribers = subscriber_rs::SubscriberCollection::<TargetInfoMessage>::new();
        target_subscribers.add(Rc::downgrade(&target_interpolator) as _);

        ProgramData{
            camera_view,
            gl_objects,
            gui_state,
            target_receiver,
            target_subscribers,
            target_interpolator,
            mount
        }
    }
}

fn create_target_mesh(
    display: &glium::Display
) -> MeshBuffers<MeshVertex> {
    use cgmath::Point3 as Point3;
    use cgmath::Vector3 as Vector3;

    // dimensions based on B737 MAX
    const LENGTH: f32 = 35.56;
    const FUSELAGE_D: f32 = 3.76;
    const NUM_FUSELAGE_SEGS: usize = 20;
    const WING_WIDTH_BASE: f32 = 6.0;
    const WING_WIDTH_END: f32 = 1.7;
    const WINGSPAN: f32 = 31.0;
    const WING_ANGLE: Deg<f32> = Deg(30.0);

    let mut vertex_data: Vec<MeshVertex> = vec![];
    let mut index_data: Vec<u32> = vec![];

    let l_half = Vector3::new(LENGTH / 2.0, 0.0, 0.0);

    // create fuselage
    for i in 0..NUM_FUSELAGE_SEGS {
        let p = FUSELAGE_D / 2.0 * Point3::from_vec(Basis3::from_angle_x(Deg(i as f32 * 360.0 / NUM_FUSELAGE_SEGS as f32))
            .rotate_vector(Vector3::unit_y()));

        vertex_data.push(MeshVertex{
            position: *(p + l_half).as_ref(),
            normal: *p.to_vec().as_ref()
        });

        vertex_data.push(MeshVertex{
            position: *(p - l_half).as_ref(),
            normal: *p.to_vec().as_ref()
        });

        index_data.push( (2 * i)                                as u32);
        index_data.push(((2 * i + 2) % (2 * NUM_FUSELAGE_SEGS)) as u32);
        index_data.push( (2 * i + 1)                            as u32);

        index_data.push(( 2 * i + 1)                            as u32);
        index_data.push(((2 * i + 2) % (2 * NUM_FUSELAGE_SEGS)) as u32);
        index_data.push(((2 * i + 3) % (2 * NUM_FUSELAGE_SEGS)) as u32);
    }

    // create wings
    let back = WINGSPAN / (2.0 * Rad::from(WING_ANGLE).0.tan());
    let p0 = Point3{ x:  WING_WIDTH_BASE / 2.0,        y: 0.0,             z: 0.0 };
    let p1 = Point3{ x: -WING_WIDTH_BASE / 2.0,        y: 0.0,             z: 0.0 };
    let p2 = Point3{ x:  -WING_WIDTH_END / 2.0 - back, y: -WINGSPAN / 2.0, z: 0.0 };
    let p3 = Point3{ x:   WING_WIDTH_END / 2.0 - back, y: -WINGSPAN / 2.0, z: 0.0 };
    let p4 = Point3{ x:  -WING_WIDTH_END / 2.0 - back, y:  WINGSPAN / 2.0, z: 0.0 };
    let p5 = Point3{ x:   WING_WIDTH_END / 2.0 - back, y:  WINGSPAN / 2.0, z: 0.0 };

    let base_idx = vertex_data.len();

    for p in [p0, p1, p2, p3, p4, p5] {
        vertex_data.push(MeshVertex{
            position: *p.as_ref(),
            normal: [0.0, 0.0, 1.0]
        });
    }

    for i in [0, 1, 3] { index_data.push((base_idx + i) as u32); }
    for i in [1, 2, 3] { index_data.push((base_idx + i) as u32); }
    for i in [0, 5, 1] { index_data.push((base_idx + i) as u32); }
    for i in [1, 5, 4] { index_data.push((base_idx + i) as u32); }

    let vertices = Rc::new(glium::VertexBuffer::new(display, &vertex_data).unwrap());
    let indices = Rc::new(glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &index_data).unwrap());

    MeshBuffers{ vertices, indices }
}

fn create_sky_mesh(
    step: cgmath::Deg<f64>,
    num_substeps: usize,
    display: &glium::Display
) -> MeshBuffers<Vertex3> {
    let mut vertex_data: Vec<Vertex3> = vec![];
    let mut index_data: Vec<u32> = vec![];

    let mut longitude = cgmath::Deg(-180.0);
    while longitude <= cgmath::Deg(180.0) {
        let mut latitude = cgmath::Deg(-90.0);
        let mut parallel_starts = true;
        while latitude <= cgmath::Deg(90.0) {
            vertex_data.push(Vertex3{
                position: *to_global_unit(&LatLon{ lat: latitude, lon: longitude }).0.cast::<f32>().unwrap().as_ref()
            });
            if !parallel_starts {
                index_data.push((vertex_data.len() - 2) as u32);
                index_data.push((vertex_data.len() - 1) as u32);
            }
            latitude += step / num_substeps as f64;
            parallel_starts = false;
        }

        longitude += step;
    }

    let mut latitude = cgmath::Deg(-90.0);
    while latitude <= cgmath::Deg(90.0) {
        let mut longitude = cgmath::Deg(-180.0);
        let mut meridian_starts = true;
        while longitude <= cgmath::Deg(180.0) {
            vertex_data.push(Vertex3{
                position: *to_global_unit(&LatLon{ lat: latitude, lon: longitude }).0.cast::<f32>().unwrap().as_ref()
            });
            if !meridian_starts {
                index_data.push((vertex_data.len() - 2) as u32);
                index_data.push((vertex_data.len() - 1) as u32);
            }
            longitude += step / num_substeps as f64;
            meridian_starts = false;
        }

        latitude += step;
    }

    let vertices = Rc::new(glium::VertexBuffer::new(display, &vertex_data).unwrap());
    let indices = Rc::new(glium::IndexBuffer::new(display, glium::index::PrimitiveType::LinesList, &index_data).unwrap());

    MeshBuffers{ vertices, indices }
}
