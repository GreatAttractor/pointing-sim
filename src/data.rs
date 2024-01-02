//
// Pointing Simulator
// Copyright (c) 2023 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use cgmath::{Basis3, Deg, EuclideanSpace, InnerSpace, Point3, Rad, Rotation, Rotation3, Vector3};
use crate::gui::CameraView;
use glium::program;
use scan_fmt::scan_fmt;
use std::{cell::RefCell, error::Error, rc::{Rc, Weak}};

/// Arithmetic mean radius (R1) as per IUGG.
pub const EARTH_RADIUS: f64 = 6_371_008.8;

/// Target information using observer's frame of reference (X points north, Z points up, Y points west).
#[derive(Debug)]
pub struct TargetInfoMessage {
    pub position: Point3<f64>,
    pub velocity: Vector3<f64>,
    pub track: Deg<f64>
}

impl std::str::FromStr for TargetInfoMessage {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y, z, vx, vy, vz, track) = scan_fmt!(s, "{};{};{};{};{};{};{}", f64, f64, f64, f64, f64, f64, f64)?;

        Ok(TargetInfoMessage{ position: Point3::new(x, y, z), velocity: Vector3::new(vx, vy, vz), track: Deg(track) })
    }
}

impl std::fmt::Display for TargetInfoMessage  {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f, "{:.1};{:.1};{:.1};{:.1};{:.1};{:.1};{:.1}\n",
            self.position.x, self.position.y, self.position.z,
            self.velocity.x, self.velocity.y, self.velocity.z,
            self.track.0
        )
    }
}

#[derive(Clone, Debug)]
pub struct LatLon {
    pub lat: Deg<f64>,
    pub lon: Deg<f64>
}

impl LatLon {
    pub fn new(lat: Deg<f64>, lon: Deg<f64>) -> LatLon {
        LatLon{ lat, lon }
    }
}

pub struct GeoPos {
    pub lat_lon: LatLon,
    pub elevation: f64
}

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
    pub target_subscribers: subscriber_rs::SubscriberCollection<TargetInfoMessage>
}

impl ProgramData {
    pub fn new(
        renderer: &Rc<RefCell<imgui_glium_renderer::Renderer>>,
        display: &glium::Display,
        gui_state: crate::gui::GuiState,
        target_receiver: crossbeam::channel::Receiver<TargetInfoMessage>
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

        let mut target_subscribers = subscriber_rs::SubscriberCollection::<TargetInfoMessage>::new();
        target_subscribers.add(Rc::downgrade(&camera_view) as _);

        ProgramData{
            camera_view,
            gl_objects,
            gui_state,
            target_receiver,
            target_subscribers
        }
    }
}

fn create_target_mesh(
    display: &glium::Display
) -> MeshBuffers<MeshVertex> {
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
                position: *to_xyz_unit(&LatLon{ lat: latitude, lon: longitude }).cast::<f32>().unwrap().as_ref()
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
                position: *to_xyz_unit(&LatLon{ lat: latitude, lon: longitude }).cast::<f32>().unwrap().as_ref()
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

/// Coordinates in Cartesian frame with lat. 0°, lon. 0° being (1, 0, 0) and the North Pole at (0, 0, 1).
pub fn to_xyz_unit(lat_lon: &LatLon) -> Point3<f64> {
    Point3{
        x: Rad::from(lat_lon.lon).0.cos() * Rad::from(lat_lon.lat).0.cos(),
        y: Rad::from(lat_lon.lon).0.sin() * Rad::from(lat_lon.lat).0.cos(),
        z: Rad::from(lat_lon.lat).0.sin()
    }
}

pub fn to_global(position: &GeoPos) -> Point3<f64> {
    let r = EARTH_RADIUS + position.elevation;
    r * to_xyz_unit(&position.lat_lon)
}

/// Converts position of `target` into `observer`s local frame (X points north, Y points west, Z points up).
pub fn to_local(observer: &GeoPos, target: &GeoPos) -> Point3<f64> {
    let obs_xyz = to_global(observer);
    let target_xyz = to_global(target);
    let local_z_axis = obs_xyz.to_vec().normalize();
    let to_north_pole = Point3::new(0.0, 0.0, EARTH_RADIUS) - obs_xyz;
    let local_y_axis = local_z_axis.cross(to_north_pole).normalize();
    let local_x_axis = local_y_axis.cross(local_z_axis);
    let to_target = target_xyz - obs_xyz;

    let x = local_x_axis.dot(to_target);
    let y = local_y_axis.dot(to_target);
    let z = local_z_axis.dot(to_target);

    Point3{ x, y, z }
}
