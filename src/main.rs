//
// Pointing Simulator
// Copyright (c) 2023 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

mod data;
mod gui;
mod runner;

fn main() {
    const DEFAULT_FONT_SIZE: f32 = 15.0;
    let runner = runner::create_runner(DEFAULT_FONT_SIZE);
    let mut data = None;
    let mut gui_state = gui::GuiState::new(runner.platform().hidpi_factor(), DEFAULT_FONT_SIZE);

    runner.main_loop(move |_, ui, display, renderer| {
        if data.is_none() {
            data = Some(data::ProgramData::new(renderer, display));
        }
        gui::handle_gui(data.as_mut().unwrap(), ui, &mut gui_state, renderer, display)
    });
}
