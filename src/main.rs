// SPDX-FileCopyrightText: 2023 Manuel Quarneti <hi@mq1.eu>
// SPDX-License-Identifier: GPL-3.0-only

use std::thread;

mod util;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let drives = util::list_drives();
    ui.set_drives(drives);

    let ui_handle = ui.as_weak();
    ui.on_open_drive(move |drive_name| {
        let ui = ui_handle.unwrap();

        let drive_path = util::open_folder(&drive_name);

        if !drive_path.is_empty() {
            let games = util::get_games(&drive_path);
            ui.set_games(games);
        }

        ui.set_drive_path(drive_path);
        ui.set_view("games".into());
    });

    let ui_handle = ui.as_weak();
    ui.on_add_games(move |drive_path| {
        let ui_handle = ui_handle.clone();

        thread::spawn(move || {
            futures_executor::block_on(async move {
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
                    util::add_game(&drive_path, &game);
                }

                let handle_weak = ui_handle.clone();
                handle_weak
                    .upgrade_in_event_loop(move |handle_weak| {
                        let games = util::get_games(&drive_path);
                        handle_weak.set_view("games".into());
                        handle_weak.set_games(games);
                    })
                    .unwrap();
            });
        });
    });

    let ui_handle = ui.as_weak();
    ui.on_remove_game(move |game| {
        let ui = ui_handle.unwrap();
        let drive_path = ui.get_drive_path();

        let games = util::remove_game(&drive_path, &game);
        ui.set_games(games);
    });

    ui.run()
}
