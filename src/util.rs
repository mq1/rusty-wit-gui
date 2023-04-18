// SPDX-FileCopyrightText: 2023 Manuel Quarneti <hi@mq1.eu>
// SPDX-License-Identifier: GPL-3.0-only

use crate::Game;

use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command, io,
};

use anyhow::Result;
use ini::Ini;
use rfd::{FileDialog, MessageButtons, MessageDialog};
use slint::{ModelRc, VecModel};

pub fn download_wit(drive_mount_point: &str) -> Result<()> {
    let file_name = if cfg!(target_os = "macos") {
        "wit-v3.04a-r8427-mac.tar.gz"
    } else if cfg!(target_os = "windows") {
        "wit-v3.04a-r8427-cygwin64.zip"
    } else {
        "wit-v3.04a-r8427-x86_64.tar.gz"
    };

    let download_path = Path::new(drive_mount_point).join(file_name);
    let download_url = format!("https://wit.wiimm.de/download/{file_name}");

    let mut body = ureq::get(&download_url).call()?.into_reader();
    let mut file = File::create(&download_path)?;
    io::copy(&mut body, &mut file)?;

    if cfg!(target_family = "unix") {
        let output = Command::new("tar")
            .arg("xzf")
            .arg(&download_path)
            .arg("-C")
            .arg(drive_mount_point)
            .output()?;
        println!("{:?}", output);
    } else {
        let output = Command::new("powershell")
            .arg("Expand-Archive")
            .arg(&download_path)
            .arg(drive_mount_point)
            .output()?;
        println!("{:?}", output);
    }

    fs::remove_file(&download_path)?;

    Ok(())
}

pub fn get_wit_path(drive_mount_point: &str) -> Result<PathBuf> {
    let base_dir = if cfg!(target_os = "macos") {
        Path::new(drive_mount_point).join("wit-v3.04a-r8427-mac")
    } else if cfg!(target_os = "windows") {
        Path::new(drive_mount_point).join("wit-v3.04a-r8427-cygwin64")
    } else {
        Path::new(drive_mount_point).join("wit-v3.04a-r8427-x86_64")
    };

    let wit_path = if cfg!(target_family = "unix") {
        base_dir.join("bin").join("wit")
    } else {
        base_dir.join("bin").join("wit.exe")
    };

    Ok(wit_path)
}

pub fn get_games(drive_mount_point: &str) -> Result<ModelRc<Game>> {
    let wbfs_folder = Path::new(drive_mount_point).join("wbfs");
    if !wbfs_folder.exists() {
        fs::create_dir(&wbfs_folder)?;
    }

    let wit_path = get_wit_path(drive_mount_point)?;
    if !wit_path.exists() {
        download_wit(drive_mount_point)?;
    }

    let output = Command::new(wit_path)
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

pub fn select_games() -> Vec<PathBuf> {
    let games = FileDialog::new().add_filter("ISO", &["iso"]).pick_files();

    match games {
        Some(games) => games,
        None => Vec::new(),
    }
}

pub fn add_game(drive_mount_point: &str, game: &Path) -> Result<()> {
    let wit_path = get_wit_path(drive_mount_point)?;

    let output = Command::new(&wit_path).arg("id6").arg(&game).output()?;
    let game_id = String::from_utf8(output.stdout)?.trim().to_string();

    let path = Path::new(drive_mount_point)
        .join("wbfs")
        .join(game_id)
        .with_extension("wbfs");

    let output = Command::new(wit_path)
        .arg("copy")
        .arg(&game)
        .arg(&path)
        .arg("--split")
        .arg("--split-size")
        .arg("4G-32K")
        .output()?;
    println!("{:?}", output);

    Ok(())
}

pub fn remove_game(drive_mount_point: &str, game: &Game) -> Result<ModelRc<Game>> {
    let yes = MessageDialog::new()
        .set_title("Remove Game")
        .set_description(&format!("Are you sure you want to remove {}?", game.title))
        .set_buttons(MessageButtons::OkCancel)
        .show();

    if yes {
        let wit_path = get_wit_path(drive_mount_point)?;

        let output = Command::new(wit_path)
            .arg("remove")
            .arg(game.path.as_str())
            .output()?;
        println!("{:?}", output);
    }

    get_games(drive_mount_point)
}
