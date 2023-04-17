// SPDX-FileCopyrightText: 2023 Manuel Quarneti <hi@mq1.eu>
// SPDX-License-Identifier: GPL-3.0-only

use crate::Game;

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use ini::Ini;
use rfd::{FileDialog, MessageButtons, MessageDialog};
use slint::{ModelRc, VecModel};

pub fn get_games(drive_mount_point: &str) -> Result<ModelRc<Game>> {
    let wbfs_folder = Path::new(drive_mount_point).join("wbfs");
    if !wbfs_folder.exists() {
        fs::create_dir(&wbfs_folder)?;
    }

    let output = Command::new("wit")
        .arg("list")
        .arg(wbfs_folder)
        .arg("--sections")
        .output()?;
    println!("{:?}", output);

    let output = String::from_utf8(output.stdout)?;
    let list = Ini::load_from_str(&output)?;

    let mut games = Vec::new();
    for (section, properties) in list.iter() {
        if let Some(section) = section {
            if section != "summary" {
                let game = Game {
                    id: properties.get("id").unwrap().to_string().into(),
                    title: properties.get("title").unwrap().to_string().into(),
                    size: format!(
                        "{:.2}",
                        properties.get("size").unwrap().parse::<f32>().unwrap() / 1073741824.
                    )
                    .into(),
                    path: properties.get("filename").unwrap().to_string().into(),
                };

                games.push(game);
            }
        }
    }

    Ok(VecModel::from_slice(&games))
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

pub fn remove_game(drive_mount_point: &str, game: &Game) -> Result<ModelRc<Game>> {
    let yes = MessageDialog::new()
        .set_title("Remove Game")
        .set_description(&format!("Are you sure you want to remove {}?", game.title))
        .set_buttons(MessageButtons::OkCancel)
        .show();

    if yes {
        let output = Command::new("wit")
            .arg("remove")
            .arg(game.path.as_str())
            .output()?;
        println!("{:?}", output);
    }

    get_games(drive_mount_point)
}
