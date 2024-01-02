//
// Pointing Simulator
// Copyright (c) 2023 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use cgmath::{Deg, Vector3, Zero};
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

    let observer_pos = GeoPos{ lat_lon: LatLon::new(Deg(0.0), Deg(0.0)), elevation: 0.0 };
    let mut target_pos = GeoPos{ lat_lon: LatLon::new(Deg(0.05), Deg(0.1)), elevation: 5000.0 };
    let track = Deg(-90.0);
    let target_airspeed = 200.0;

    loop {
        stream.write_all(TargetInfoMessage{
            position: data::to_local(&observer_pos, &target_pos),
            velocity: Vector3::zero(),
            track
        }.to_string().as_bytes()).unwrap();

        std::thread::sleep(MSG_DELTA_T);
    }
}
