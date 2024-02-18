//
// Pointing Simulator
// Copyright (c) 2023-2024 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use cgmath::{Basis3, Deg, EuclideanSpace, InnerSpace, Rad, Rotation, Rotation3};
use pointing_utils::{
    EARTH_RADIUS_M,
    GeoPos,
    Global,
    LatLon,
    Point3,
    TargetInfoMessage,
    to_global,
    to_local_point,
    to_local_vec,
    Vector3,
    uom
};
use std::{io::Write, net::{TcpListener, TcpStream}, sync::{Arc, Mutex}};
use uom::{si::f64, si::length};

const MSG_DELTA_T: std::time::Duration = std::time::Duration::from_millis(250);

pub const TARGET_SOURCE_PORT: u16 = 45500;

fn meters(value: f64) -> f64::Length {
    f64::Length::new::<length::meter>(value)
}

pub fn target_source() {
    type P3G = Point3<f64, Global>;
    type V3G = Vector3<f64, Global>;

    let clients = Arc::new(Mutex::new(Vec::<TcpStream>::new()));

    let clients2 = Arc::clone(&clients);
    std::thread::spawn(move || {
        log::info!("waiting for clients");
        let listener = TcpListener::bind(format!("127.0.0.1:{}", TARGET_SOURCE_PORT)).unwrap();
        loop {
            let (stream, _) = listener.accept().unwrap();
            log::info!("client connected");
            clients2.lock().unwrap().push(stream);
        }
    });

    let observer_pos = to_global(&GeoPos{ lat_lon: LatLon::new(Deg(0.0), Deg(0.0)), elevation: meters(0.0) });
    let target_elevation = meters(5000.0);
    let target_initial_pos = GeoPos{ lat_lon: LatLon::new(Deg(0.05), Deg(0.1)), elevation: target_elevation };
    let mut target_pos = to_global(&target_initial_pos);
    let north_pole = Point3::<f64, Global>::from_xyz(0.0, 0.0, EARTH_RADIUS_M);

    let track = Deg(-90.0);
    let target_speed = 200.0;

    let mut t_last_update = std::time::Instant::now();
    loop {
        // assume level flight
        let arc_length = t_last_update.elapsed().as_secs_f64() * target_speed;
        let travel_angle = Rad(arc_length / (EARTH_RADIUS_M + target_elevation.get::<length::meter>()));
        let to_north_pole = V3G::from(north_pole.0 - target_pos.0);
        let west = V3G::from(target_pos.0.to_vec().cross(to_north_pole.0));
        let north = V3G::from(west.0.cross(target_pos.0.to_vec()).normalize());
        let track_dir = V3G::from(Basis3::from_axis_angle(target_pos.0.to_vec().normalize(), -track).rotate_vector(north.0));
        let fwd_axis = V3G::from(target_pos.0.to_vec().cross(track_dir.0).normalize());
        target_pos = P3G::from(Basis3::from_axis_angle(fwd_axis.0, travel_angle).rotate_point(target_pos.0));
        t_last_update = std::time::Instant::now();

        clients.lock().unwrap().retain_mut(|client| {
            match client.write_all(TargetInfoMessage{
                position: to_local_point(&observer_pos, &target_pos),
                velocity: to_local_vec(&observer_pos, &V3G::from(track_dir.0 * target_speed)),
                track,
                altitude: target_elevation
            }.to_string().as_bytes()) {

                Ok(()) => true,
                Err(e) => {
                    log::info!("error sending data ({}), disconnecting from client", e);
                    false
                }
            }
        });

        std::thread::sleep(MSG_DELTA_T);
    }
}
