//
// Pointing Simulator
// Copyright (c) 2023-2024 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

mod data;
mod gui;
mod runner;
mod target_interpolator;
mod workers;

use crossbeam::channel::TryRecvError;

fn main() {
    let tz_offset = chrono::Local::now().offset().clone();
    simplelog::SimpleLogger::init(
        simplelog::LevelFilter::Debug,
        simplelog::ConfigBuilder::new()
            .set_target_level(simplelog::LevelFilter::Error)
            .set_time_offset(time::UtcOffset::from_whole_seconds(tz_offset.local_minus_utc()).unwrap())
            .set_time_format_custom(simplelog::format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:6]"
            ))
            .build(),
    ).unwrap();

    const DEFAULT_FONT_SIZE: f32 = 15.0;
    let runner = runner::create_runner(DEFAULT_FONT_SIZE);
    let mut data = None;
    let mut gui_state = Some(gui::GuiState::new(runner.platform().hidpi_factor(), DEFAULT_FONT_SIZE));

    runner.main_loop(move |_, ui, display, renderer| {
        if data.is_none() {
            let (sender_worker, receiver_main) = crossbeam::channel::unbounded();

            std::thread::spawn(|| { workers::target_source() });
            std::thread::spawn(move || { workers::target_receiver(sender_worker) });

            data = Some(data::ProgramData::new(renderer, display, gui_state.take().unwrap(), receiver_main));
        }

        match data.as_ref().unwrap().target_receiver.try_recv() {
            Ok(msg) => data.as_mut().unwrap().target_subscribers.notify(&msg),
            Err(e) => match e {
                TryRecvError::Empty => (),
                _ => panic!("unexpected error: {}", e)
            }
        }

        data.as_ref().unwrap().target_interpolator.borrow_mut().interpolate();

        gui::handle_gui(data.as_mut().unwrap(), ui, renderer, display)
    });
}
