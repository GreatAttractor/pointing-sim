//
// Pointing Simulator
// Copyright (c) 2023 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use std::{cell::RefCell, rc::{Rc, Weak}};

pub trait Subscriber<T> {
    fn notify(&mut self, value: &T);
}

pub struct SubscriberCollection<T> {
    subscribers: Vec<Weak<RefCell<dyn Subscriber<T>>>>
}

// not using #[derive(Default)], as it (needlessly) imposes `Default` also on `T`
impl<T> Default for SubscriberCollection<T> {
    fn default() -> SubscriberCollection<T> {
        SubscriberCollection{ subscribers: vec![] }
    }
}

impl<T> SubscriberCollection<T> {
    pub fn new() -> SubscriberCollection<T> {
        SubscriberCollection{ subscribers: vec![] }
    }

    /// Notifies all still existing subscribers; removes those no longer available.
    pub fn notify(&mut self, value: &T) {
        self.subscribers.retain_mut(|subscriber| {
            match subscriber.upgrade() {
                Some(subscriber) => {
                    subscriber.borrow_mut().notify(value);
                    true
                },

                None => false
            }
        });
    }

    pub fn add(&mut self, subscriber: Weak<RefCell<dyn Subscriber<T>>>) {
        self.subscribers.push(subscriber);
    }

    fn has_subscriber(&self, subscriber: &Rc<RefCell<dyn Subscriber<T>>>) -> bool {
        self.subscribers.iter().find(
            |s| if let Some(s) = s.upgrade() {
                // Cannot use `Rc::as_ptr()`, because it compares fat pointers (data + vtable), and the vtable may be
                // different depending on when & where the `subscriber` trait object was created (e.g., in which crate).
                // Instead, cast the fat pointers to thin pointers `*const ()` (i.e., data only).
                s.as_ptr() as *const () == subscriber.as_ptr() as *const ()
            } else {
                false
            }
        ).is_some()
    }
}
