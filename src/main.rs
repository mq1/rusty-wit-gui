// SPDX-FileCopyrightText: 2023 Manuel Quarneti <hi@mq1.eu>
// SPDX-License-Identifier: GPL-3.0-only

use std::thread;

use anyhow::Result;

mod drives;
mod util;

slint::include_modules!();

fn main() -> Result<()> {
    let ui = AppWindow::new()?;

    let window_title = env!("CARGO_PKG_NAME").to_owned() + " v" + env!("CARGO_PKG_VERSION");
    ui.set_title_(window_title.into());

    let drives = drives::list()?;
    ui.set_drives(drives);

    let ui_handle = ui.as_weak();
    ui.on_open_drive(move |drive| {
        let ui_handle = ui_handle.clone();

        thread::spawn(move || {
            let wit_path = util::get_wit_path(&drive.mount_point).unwrap();

            if !wit_path.exists() {
                util::download_wit(&drive.mount_point, &ui_handle).unwrap();
            }

            let handle_weak = ui_handle.clone();
            handle_weak
                .upgrade_in_event_loop(move |handle_weak| {
                    let games: slint::ModelRc<Game> = util::get_games(&drive.mount_point).unwrap();

                    handle_weak.set_games(games);
                    handle_weak.set_selected_drive(drive);
                    handle_weak.set_view("games".into());
                })
                .unwrap();
        });
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
                    handle_weak.set_view("progress".into());
                })
                .unwrap();

            for (i, game) in games.iter().enumerate() {
                let handle_weak = ui_handle.clone();
                handle_weak
                    .upgrade_in_event_loop(move |handle_weak| {
                        let text = format!("Adding game {}/{}", i + 1, games_count);
                        handle_weak.set_progress_text(text.into());
                    })
                    .unwrap();
                util::add_game(&drive.mount_point, &game).unwrap();
            }

            let handle_weak = ui_handle.clone();
            handle_weak
                .upgrade_in_event_loop(move |handle_weak| {
                    let games = util::get_games(&drive.mount_point).unwrap();
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

        let games = util::remove_game(&drive.mount_point, &game).unwrap();
        let drive = drives::refresh(drive).unwrap();

        ui.set_selected_drive(drive);
        ui.set_games(games);
    });

    ui.run()?;
    Ok(())
}
