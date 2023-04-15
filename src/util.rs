// SPDX-FileCopyrightText: 2023 Manuel Quarneti <hi@mq1.eu>
// SPDX-License-Identifier: GPL-3.0-only

use crate::Drive;

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use rfd::{FileDialog, MessageButtons, MessageDialog};
use slint::{ModelRc, SharedString, VecModel};
use sysinfo::{DiskExt, System, SystemExt};

pub fn list_drives() -> ModelRc<Drive> {
    let mut sys = System::new();
    sys.refresh_disks_list();

    let drives = sys
        .disks()
        .iter()
        .filter(|disk| disk.is_removable())
        .map(|disk| {
            let name = disk.name().to_string_lossy().to_string();
            let total_space_gib = disk.total_space() as f32 / 1024. / 1024. / 1024.;
            let available_space_gib = disk.available_space() as f32 / 1024. / 1024. / 1024.;
            let path = disk.mount_point().to_string_lossy().to_string();

            Drive {
                name: name.into(),
                total_space: format!("{:.2} GiB", total_space_gib).into(),
                available_space: format!("{:.2} GiB", available_space_gib).into(),
                path: path.into(),
            }
        })
        .collect::<Vec<_>>();

    VecModel::from_slice(&drives)
}

pub fn get_games(drive_path: &str) -> ModelRc<SharedString> {
    let wbfs_folder = Path::new(drive_path).join("wbfs");
    if !wbfs_folder.exists() {
        fs::create_dir(&wbfs_folder).unwrap();
    }

    let files = fs::read_dir(&wbfs_folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
        .filter(|file| file.ends_with(".wbfs"))
        .map(|file| file.into())
        .collect::<Vec<SharedString>>();

    VecModel::from_slice(&files)
}

fn get_titles(drive_path: &str) {
    let titles_path = Path::new(drive_path).join("titles.txt");

    if !titles_path.exists() {
        let titles = ureq::get("https://gametdb.com/titles.txt")
            .call()
            .unwrap()
            .into_string()
            .unwrap();

        fs::write(&titles_path, titles).unwrap();
    }
}

pub fn select_games() -> Vec<PathBuf> {
    let games = FileDialog::new().add_filter("ISO", &["iso"]).pick_files();

    match games {
        Some(games) => games,
        None => Vec::new(),
    }
}

pub fn add_game(drive_path: &str, game: &Path) {
    let wbfs_folder = Path::new(drive_path).join("wbfs");
    let titles_path = Path::new(drive_path).join("titles.txt");

    get_titles(drive_path);

    let output = Command::new("wit").arg("id6").arg(&game).output().unwrap();
    let game_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    for line in fs::read_to_string(&titles_path).unwrap().lines() {
        let parts = line.split(" = ").collect::<Vec<_>>();
        let id = parts[0];
        let title = parts[1];

        if id == &game_id {
            let game_name = format!("{title} [{id}].wbfs");
            let game_path = wbfs_folder.join(&game_name);

            Command::new("wit")
                .arg("copy")
                .arg(&game)
                .arg(&game_path)
                .output()
                .unwrap();

            let game_meta = format!("._{game_name}");
            let game_meta_path = wbfs_folder.join(game_meta);
            fs::remove_file(game_meta_path).unwrap();

            break;
        }
    }
}

pub fn remove_game(drive_path: &str, game: &str) -> ModelRc<SharedString> {
    let yes = MessageDialog::new()
        .set_title("Remove Game")
        .set_description(&format!("Are you sure you want to remove {game}?"))
        .set_buttons(MessageButtons::OkCancel)
        .show();

    if yes {
        let wbfs_folder = Path::new(drive_path).join("wbfs");
        let game_path = wbfs_folder.join(game);

        fs::remove_file(game_path).unwrap();
    }

    get_games(drive_path)
}
