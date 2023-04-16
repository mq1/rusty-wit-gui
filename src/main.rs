// SPDX-FileCopyrightText: 2023 Manuel Quarneti <hi@mq1.eu>
// SPDX-License-Identifier: GPL-3.0-only

use std::thread;

use anyhow::Result;

mod util;
mod drives;

slint::include_modules!();

fn main() -> Result<()> {
    let ui = AppWindow::new()?;

    let window_title = env!("CARGO_PKG_NAME").to_owned() + " v" + env!("CARGO_PKG_VERSION");
    ui.set_title_(window_title.into());

    let drives = drives::list()?;
    ui.set_drives(drives);

    let ui_handle = ui.as_weak();
    ui.on_open_drive(move |drive| {
        let ui = ui_handle.unwrap();
        let games = util::get_games(&drive.mount_point);

        ui.set_games(games.unwrap());
        ui.set_selected_drive(drive);
        ui.set_view("games".into());
    });

    let ui_handle = ui.as_weak();
    ui.on_format_drive(move |drive| {
        let ui = ui_handle.unwrap();
        drives::format(&drive).unwrap();

        let drives = drives::list().unwrap();
        ui.set_drives(drives);
    });

    let ui_handle = ui.as_weak();
    ui.on_add_games(move |drive| {
        let ui_handle = ui_handle.clone();

        thread::spawn(move || {
            let games = util::select_games();
            let games_count = games.len() as i32;

            let handle_weak = ui_handle.clone();
            handle_weak
                .upgrade_in_event_loop(move |handle_weak| {
                    handle_weak.set_view("adding-games".into());
                    handle_weak.set_max_progress(games_count);
                })
                .unwrap();

            for (i, game) in games.iter().enumerate() {
                let handle_weak = ui_handle.clone();
                handle_weak
                    .upgrade_in_event_loop(move |handle_weak| {
                        handle_weak.set_current_progress(i as i32 + 1);
                    })
                    .unwrap();
                util::add_game(&drive.path, &game).unwrap();
            }

            let handle_weak = ui_handle.clone();
            handle_weak
                .upgrade_in_event_loop(move |handle_weak| {
                    let games = util::get_games(&drive.path).unwrap();
                    let drive = drives::refresh(drive).unwrap();

                    handle_weak.set_selected_drive(drive);
                    handle_weak.set_view("games".into());
                    handle_weak.set_games(games);
                })
                .unwrap();
        });
    });

    let ui_handle = ui.as_weak();
    ui.on_remove_game(move |game| {
        let ui = ui_handle.unwrap();
        let drive = ui.get_selected_drive();

        let games = util::remove_game(&drive.path, &game).unwrap();
        let drive = drives::refresh(drive).unwrap();

        ui.set_selected_drive(drive);
        ui.set_games(games);
    });

    ui.run()?;
    Ok(())
}
