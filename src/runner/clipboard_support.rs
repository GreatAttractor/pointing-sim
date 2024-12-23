//
// Pointing Simulator
// Copyright (c) 2023 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use clipboard::{ClipboardContext, ClipboardProvider};
use imgui::ClipboardBackend;

pub struct ClipboardSupport(ClipboardContext);

pub fn init() -> Option<ClipboardSupport> {
    ClipboardContext::new()
        .ok()
        .map(|ctx| ClipboardSupport(ctx))
}

impl ClipboardBackend for ClipboardSupport {
    fn get(&mut self) -> Option<String> {
        self.0.get_contents().ok().map(|text| text.into())
    }
    fn set(&mut self, text: &str) {
        let _ = self.0.set_contents(text.to_string());
    }
}
