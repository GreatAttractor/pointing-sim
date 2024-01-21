use pointing_utils::{MountSimulatorMessage, read_line, uom};
use std::{io::Write, net::TcpListener, sync::{Arc, RwLock}};
use uom::{si::f64, si::{angle, angular_acceleration, angular_velocity, time}};

pub const MOUNT_SERVER_PORT: u16 = 45501;
// TODO: replace with const `angular_acceleration::degree_per_second_squared` once supported
const AXIS_ANG_ACCELERATION: f64 = 6.0;

mod axis {
    use super::*;
    pub struct Axis {
        t0: std::time::Instant,
        pos0: f64::Angle,
        spd0: f64::AngularVelocity,
        target_spd: f64::AngularVelocity,
        accel_dt: f64::Time,
    }

    impl Axis {
        pub fn new(pos: f64::Angle, speed: f64::AngularVelocity) -> Axis {
            Axis{
                t0: std::time::Instant::now(),
                pos0: pos,
                spd0: speed,
                target_spd: speed,
                accel_dt: time(std::time::Duration::from_secs(0))
            }
        }

        pub fn state(&self) -> (f64::Angle, f64::AngularVelocity) {
            let dt = time(self.t0.elapsed());

            let accel_sign = (self.target_spd - self.spd0).get::<angular_velocity::degree_per_second>().signum();
            let accel = accel_sign * deg_per_s_sq(AXIS_ANG_ACCELERATION);

            let speed = if dt < self.accel_dt {
                self.spd0 + Into::<f64::AngularVelocity>::into(dt * accel)
            } else {
                self.target_spd
            };

            let pos_during_accel = |dt| {
                self.pos0 + Into::<f64::Angle>::into(self.spd0 * dt) + Into::<f64::Angle>::into(accel * dt * dt / 2.0)
            };

            let pos = if dt < self.accel_dt {
                pos_during_accel(dt)
            } else {
                pos_during_accel(self.accel_dt) + Into::<f64::Angle>::into((dt - self.accel_dt) * self.target_spd)
            };

            (pos, speed)
        }

        pub fn set_target_speed(&mut self, target_spd: f64::AngularVelocity) {
            let (pos0, spd0) = self.state();

            self.t0 = std::time::Instant::now();
            self.pos0 = pos0;
            self.spd0 = spd0;
            self.target_spd = target_spd;
            self.accel_dt = (self.target_spd - self.spd0).abs() / deg_per_s_sq(AXIS_ANG_ACCELERATION);
        }
    }
}
use axis::Axis;

pub struct MountState {
    pub axis1_pos: f64::Angle,
    pub axis2_pos: f64::Angle,
    pub axis1_spd: f64::AngularVelocity,
    pub axis2_spd: f64::AngularVelocity,
}

struct PrivState {
    axis1: Axis,
    axis2: Axis
}

impl PrivState {
    pub fn new() -> PrivState {
        PrivState {
            axis1: Axis::new(deg(0.0), deg_per_s(0.0)),
            axis2: Axis::new(deg(0.0), deg_per_s(0.0)),
        }
    }
}

pub struct Mount {
    priv_state: RwLock<PrivState>
}

impl Mount {
    pub fn new() -> Mount {
        Mount{ priv_state: RwLock::new(PrivState::new()) }
    }

    pub fn get(&self) -> MountState {
        let priv_state = self.priv_state.read().unwrap();
        let (axis1_pos, axis1_spd) = priv_state.axis1.state();
        let (axis2_pos, axis2_spd) = priv_state.axis2.state();
        MountState{ axis1_pos, axis2_pos, axis1_spd, axis2_spd }
    }
}

fn time(duration: std::time::Duration) -> f64::Time { f64::Time::new::<time::second>(duration.as_secs_f64()) }

fn deg(value: f64) -> f64::Angle { f64::Angle::new::<angle::degree>(value) }

fn deg_per_s(value: f64) -> f64::AngularVelocity {
    f64::AngularVelocity::new::<angular_velocity::degree_per_second>(value)
}

fn deg_per_s_sq(value: f64) -> f64::AngularAcceleration {
    f64::AngularAcceleration::new::<angular_acceleration::degree_per_second_squared>(value)
}

// TODO: allow connecting&disconnecting more than once
pub fn mount_model(mount: Arc<Mount>) {
    type Msg = MountSimulatorMessage;

    log::info!("waiting for client");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", MOUNT_SERVER_PORT)).unwrap();
    let (mut stream, _) = listener.accept().unwrap();
    log::info!("client connected");

    loop {
        let msg_s = read_line(&mut stream).unwrap();
        match msg_s.parse::<Msg>() {
            Err(e) => log::error!("error parsing mount message: {}", e),

            Ok(msg) => match msg {
                Msg::GetPosition => {
                    let state = mount.get();
                    stream.write_all(
                        &Msg::Position(Ok((state.axis1_pos, state.axis2_pos))).to_string().as_bytes()
                    ).unwrap()
                },

                Msg::Slew{axis1, axis2} => {
                    {
                        let mut state = mount.priv_state.write().unwrap();
                        state.axis1.set_target_speed(axis1);
                        state.axis2.set_target_speed(axis2);
                    }
                    stream.write_all(&Msg::Reply(Ok(())).to_string().as_bytes()).unwrap();
                },

                Msg::Stop => {
                    {
                        let mut state = mount.priv_state.write().unwrap();
                        state.axis1.set_target_speed(deg_per_s(0.0));
                        state.axis2.set_target_speed(deg_per_s(0.0));
                    }
                    stream.write_all(&Msg::Reply(Ok(())).to_string().as_bytes()).unwrap();
                },

                _ => log::error!("unexpected message: {}", msg_s)
            }
        }
    }
}
