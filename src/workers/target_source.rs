//
// Pointing Simulator
// Copyright (c) 2023-2024 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use cgmath::{Basis3, Deg, EuclideanSpace, InnerSpace, Rad, Rotation, Rotation3, Zero};
use crate::{data, data::{GeoPos, Global, LatLon, Local, Point3, TargetInfoMessage, Vector3}};
use std::io::Write;
use std::net::TcpListener;

const MSG_DELTA_T: std::time::Duration = std::time::Duration::from_millis(1000);

pub const TARGET_SOURCE_PORT: u16 = 26262;

pub fn target_source() {
    type P3G = Point3<f64, Global>;
    type V3G = Vector3<f64, Global>;
    log::info!("waiting for client connection");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", TARGET_SOURCE_PORT)).unwrap();
    let mut stream = listener.incoming().next().unwrap().unwrap();
    log::info!("client connected");

    let observer_pos = data::to_global(&GeoPos{ lat_lon: LatLon::new(Deg(0.0), Deg(0.0)), elevation: 0.0 });
    let target_elevation = 5000.0;
    let target_initial_pos = GeoPos{ lat_lon: LatLon::new(Deg(0.05), Deg(0.1)), elevation: target_elevation };
    let mut target_pos = data::to_global(&target_initial_pos);
    let north_pole = Point3::<f64, Global>::from_xyz(0.0, 0.0, data::EARTH_RADIUS);

    let track = Deg(-90.0);
    let target_speed = 200.0;

    let mut t_last_update = std::time::Instant::now();
    loop {
        // assume level flight
        let arc_length = t_last_update.elapsed().as_secs_f64() * target_speed;
        let travel_angle = Rad(arc_length / (data::EARTH_RADIUS + target_elevation));
        let to_north_pole = V3G::from(north_pole.0 - target_pos.0);
        let west = V3G::from(target_pos.0.to_vec().cross(to_north_pole.0));
        let north = V3G::from(west.0.cross(target_pos.0.to_vec()).normalize());
        let track_dir = V3G::from(Basis3::from_axis_angle(target_pos.0.to_vec().normalize(), -track).rotate_vector(north.0));
        let fwd_axis = V3G::from(target_pos.0.to_vec().cross(track_dir.0).normalize());
        target_pos = P3G::from(Basis3::from_axis_angle(fwd_axis.0, travel_angle).rotate_point(target_pos.0));
        t_last_update = std::time::Instant::now();

        stream.write_all(TargetInfoMessage{
            position: data::to_local_point(&observer_pos, &target_pos),
            velocity: data::to_local_vec(&observer_pos, &V3G::from(track_dir.0 * target_speed)),
            track
        }.to_string().as_bytes()).unwrap();


        std::thread::sleep(MSG_DELTA_T);
    }
}
