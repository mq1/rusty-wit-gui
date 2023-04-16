// SPDX-FileCopyrightText: 2023 Manuel Quarneti <hi@mq1.eu>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use rfd::{FileDialog, MessageButtons, MessageDialog};
use slint::{ModelRc, SharedString, VecModel};

pub fn get_games(drive_mount_point: &str) -> Result<ModelRc<SharedString>> {
    let wbfs_folder = Path::new(drive_mount_point).join("wbfs");
    if !wbfs_folder.exists() {
        fs::create_dir(&wbfs_folder)?;
    }

    let files = fs::read_dir(&wbfs_folder)?
        .map(|entry| Ok(entry?.file_name().to_string_lossy().to_string()))
        .filter(|file| {
            file.as_ref()
                .map(|file| file.ends_with(".wbfs"))
                .unwrap_or(false)
        })
        .map(|file| file.map(|file| file.into()))
        .collect::<Result<Vec<SharedString>>>()?;

    Ok(VecModel::from_slice(&files))
}

fn get_titles(drive_mount_point: &str) -> Result<()> {
    let titles_path = Path::new(drive_mount_point).join("titles.txt");

    if !titles_path.exists() {
        let titles = ureq::get("https://gametdb.com/titles.txt")
            .call()?
            .into_string()?;

        fs::write(&titles_path, titles)?;
    }

    Ok(())
}

pub fn select_games() -> Vec<PathBuf> {
    let games = FileDialog::new().add_filter("ISO", &["iso"]).pick_files();

    match games {
        Some(games) => games,
        None => Vec::new(),
    }
}

pub fn add_game(drive_mount_point: &str, game: &Path) -> Result<()> {
    let wbfs_folder = Path::new(drive_mount_point).join("wbfs");
    let titles_path = Path::new(drive_mount_point).join("titles.txt");

    get_titles(drive_mount_point)?;

    let output = Command::new("wit").arg("id6").arg(&game).output()?;
    let game_id = String::from_utf8(output.stdout)?.trim().to_string();

    for line in fs::read_to_string(&titles_path)?.lines() {
        let parts = line.split(" = ").collect::<Vec<_>>();
        let id = parts[0];
        let title = parts[1];

        if id == &game_id {
            let game_name = format!("{title} [{id}].wbfs");
            let game_path = wbfs_folder.join(&game_name);

            let output = Command::new("wit")
                .arg("copy")
                .arg(&game)
                .arg(&game_path)
                .output()?;
            println!("{:?}", output);

            let game_meta = format!("._{game_name}");
            let game_meta_path = wbfs_folder.join(game_meta);
            fs::remove_file(game_meta_path)?;

            break;
        }
    }

    Ok(())
}

pub fn remove_game(drive_mount_point: &str, game: &str) -> Result<ModelRc<SharedString>> {
    let yes = MessageDialog::new()
        .set_title("Remove Game")
        .set_description(&format!("Are you sure you want to remove {game}?"))
        .set_buttons(MessageButtons::OkCancel)
        .show();

    if yes {
        let wbfs_folder = Path::new(drive_mount_point).join("wbfs");
        let game_path = wbfs_folder.join(game);

        fs::remove_file(game_path)?;
    }

    get_games(drive_mount_point)
}
