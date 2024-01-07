//
// Pointing Simulator
// Copyright (c) 2023-2024 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use crate::workers;
use pointing_utils::TargetInfoMessage;
use std::{
    io::BufRead,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream}
};

pub fn target_receiver(sender: crossbeam::channel::Sender<TargetInfoMessage>) {
    let stream;
    loop {
        if let Ok(s) = TcpStream::connect_timeout(
            &SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), workers::target_source::TARGET_SOURCE_PORT),
            std::time::Duration::from_millis(50)
        ) {
            stream = s;
            break;
        }
    }

    let buf_reader = std::io::BufReader::new(stream);

    for message in buf_reader.lines() {
        let _ = sender.send(message.unwrap().parse::<TargetInfoMessage>().unwrap());
    }
}
