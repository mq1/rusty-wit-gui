// SPDX-FileCopyrightText: 2023 Manuel Quarneti <hi@mq1.eu>
// SPDX-License-Identifier: GPL-3.0-only

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
    ui.on_add_games(move || {
        let ui = ui_handle.unwrap();
        let drive_path = ui.get_drive_path();

        let games = util::select_games();
        ui.set_view("adding-games".into());
        ui.set_max_progress(games.len() as i32);
        for (i, game) in games.iter().enumerate() {
            ui.set_current_progress(i as i32 + 1);
            // TODO: refresh ui
            util::add_game(&drive_path, &game);
        }

        let games = util::get_games(&drive_path);
        ui.set_games(games);
        ui.set_view("games".into());
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
