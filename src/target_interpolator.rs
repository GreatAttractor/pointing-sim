//
// Pointing Simulator
// Copyright (c) 2024 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use crate::data::{Local, Point3, Vector3, TargetInfoMessage};
use std::{cell::RefCell, rc::Weak};
use subscriber_rs::{Subscriber, SubscriberCollection};

struct Interpolated {
    position: Point3<f64, Local>,
    velocity: Vector3<f64, Local>,
}

pub struct TargetInterpolator {
    last_info: Option<(std::time::Instant, TargetInfoMessage)>,
    interpolated: Option<Interpolated>,
    subscribers: SubscriberCollection<TargetInfoMessage>
}

impl TargetInterpolator {
    pub fn new() -> TargetInterpolator {
        TargetInterpolator{
            last_info: None,
            interpolated: None,
            subscribers: Default::default()
        }
    }

    pub fn add_subscriber(&mut self, subscriber: Weak<RefCell<dyn Subscriber<TargetInfoMessage>>>) {
        self.subscribers.add(subscriber as _);
    }

    pub fn interpolate(&mut self) {
        if let Some(last_info) = &self.last_info {
            let dt = last_info.0.elapsed();
            let interpolated = Interpolated{
                position: Point3::<f64, Local>::from(last_info.1.position.0 + last_info.1.velocity.0 * dt.as_secs_f64()),
                velocity: last_info.1.velocity.clone()
            };
            self.subscribers.notify(&TargetInfoMessage{
                position: interpolated.position.clone(),
                velocity: interpolated.velocity.clone(),
                track: last_info.1.track
            });
            self.interpolated = Some(interpolated);
        }
    }
}

impl Subscriber<TargetInfoMessage> for TargetInterpolator {
    fn notify(&mut self, value: &TargetInfoMessage) {
        self.last_info = Some((std::time::Instant::now(), value.clone()));
        self.interpolated = Some(Interpolated{ position: value.position.clone(), velocity: value.velocity.clone() });
        self.subscribers.notify(value);
    }
}
