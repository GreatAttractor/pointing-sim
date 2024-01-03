//
// Pointing Simulator
// Copyright (c) 2023 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use cgmath::{Basis3, Deg, EuclideanSpace, InnerSpace, Point3, Rad, Rotation, Rotation3, Vector3, Zero};
use crate::{data, data::{GeoPos, LatLon, TargetInfoMessage}};
use std::io::Write;
use std::net::TcpListener;

const MSG_DELTA_T: std::time::Duration = std::time::Duration::from_millis(1000);

pub const TARGET_SOURCE_PORT: u16 = 26262;

pub fn target_source() {
    log::info!("waiting for client connection");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", TARGET_SOURCE_PORT)).unwrap();
    let mut stream = listener.incoming().next().unwrap().unwrap();
    log::info!("client connected");

    let observer_pos = data::to_global(&GeoPos{ lat_lon: LatLon::new(Deg(0.0), Deg(0.0)), elevation: 0.0 });
    let target_elevation = 5000.0;
    let target_initial_pos = GeoPos{ lat_lon: LatLon::new(Deg(0.05), Deg(0.1)), elevation: target_elevation };
    let mut target_pos = data::to_global(&target_initial_pos);
    let north_pole = Point3::new(0.0, 0.0, data::EARTH_RADIUS);

    let track = Deg(-90.0);
    let target_speed = 200.0;

    let mut t_last_update = std::time::Instant::now();
    loop {
        // assume level flight
        let arc_length = t_last_update.elapsed().as_secs_f64() * target_speed;
        let travel_angle = Rad(arc_length / (data::EARTH_RADIUS + target_elevation));
        let to_north_pole = north_pole - target_pos;
        let west = target_pos.to_vec().cross(to_north_pole);
        let north = west.cross(target_pos.to_vec()).normalize();
        let track_dir = Basis3::from_axis_angle(target_pos.to_vec().normalize(), -track).rotate_vector(north);
        let fwd_axis = target_pos.to_vec().cross(track_dir).normalize();
        target_pos = Basis3::from_axis_angle(fwd_axis, travel_angle).rotate_point(target_pos);
        t_last_update = std::time::Instant::now();

        stream.write_all(TargetInfoMessage{
            position: data::to_local_from_global(&observer_pos, &target_pos),
            velocity: Vector3::zero(),
            track
        }.to_string().as_bytes()).unwrap();


        std::thread::sleep(MSG_DELTA_T);
    }
}
