//
// Pointing Simulator
// Copyright (c) 2023 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use std::io::Write;
use std::net::TcpListener;

const MSG_DELTA_T: std::time::Duration = std::time::Duration::from_millis(1000);

pub const TARGET_SOURCE_PORT: u16 = 26262;

pub fn target_source() {
    log::info!("waiting for client connection");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", TARGET_SOURCE_PORT)).unwrap();
    let mut stream = listener.incoming().next().unwrap().unwrap();
    log::info!("client connected");
    let mut counter = 0;
    loop {
        stream.write_all(format!("message {}\n", counter).as_bytes()).unwrap();
        counter += 1;

        std::thread::sleep(MSG_DELTA_T);
    }
}
