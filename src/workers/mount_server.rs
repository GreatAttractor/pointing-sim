use pointing_utils::{MountSimulatorMessage, read_line, uom};
use std::{error::Error, io::{Read, Write}, net::{TcpListener, TcpStream}, sync::{Arc, Mutex}};
use uom::{si::f64, si::{angle, angular_velocity}};

pub const MOUNT_SERVER_PORT: u16 = 45501;

struct State {
    axis1_pos: f64::Angle,
    axis2_pos: f64::Angle,
    axis1_spd: f64::AngularVelocity,
    axis2_spd: f64::AngularVelocity,
}

fn deg(value: f64) -> f64::Angle { f64::Angle::new::<angle::degree>(value) }

fn deg_per_s(value: f64) -> f64::AngularVelocity {
    f64::AngularVelocity::new::<angular_velocity::degree_per_second>(value)
}

pub enum MountMessage {
    AxesPosition{ axis1: f64::Angle, axis2: f64::Angle}
}

// TODO: allow connecting&disconnecting more than once
pub fn mount_server(sender: crossbeam::channel::Sender<MountMessage>) {
    type Msg = MountSimulatorMessage;

    let mut state = State{
        axis1_pos: deg(0.0),
        axis2_pos: deg(0.0),
        axis1_spd: deg_per_s(0.0),
        axis2_spd: deg_per_s(0.0)
    };

    sender.send(MountMessage::AxesPosition{ axis1: state.axis1_pos, axis2: state.axis2_pos }).unwrap();

    log::info!("waiting for client");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", MOUNT_SERVER_PORT)).unwrap();
    let (mut stream, _) = listener.accept().unwrap();
    log::info!("client connected");

    loop {
        let msg_s = read_line(&mut stream).unwrap();
        log::info!("received: {}", msg_s); //TESTING ##########
        match msg_s.parse::<Msg>() {
            Err(e) => log::error!("error parsing mount message: {}", e),
            Ok(msg) => match msg {
                Msg::GetPosition => stream.write_all(
                    &Msg::Position(Ok((state.axis1_pos, state.axis2_pos))).to_string().as_bytes()
                ).unwrap(),

                Msg::Slew{axis1, axis2} => {
                    state.axis1_spd = axis1;
                    state.axis2_spd = axis2;
                    stream.write_all(&Msg::Reply(Ok(())).to_string().as_bytes()).unwrap();
                },

                Msg::Stop => {
                    state.axis1_spd = deg_per_s(0.0);
                    state.axis2_spd = deg_per_s(0.0);
                    stream.write_all(&Msg::Reply(Ok(())).to_string().as_bytes()).unwrap();
                },

                _ => log::error!("unexpected message: {}", msg_s)
            }
        }
    }
}
