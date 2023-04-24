// SPDX-FileCopyrightText: 2023 Manuel Quarneti <hi@mq1.eu>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs,
    io::{BufReader, Cursor, Read},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use flate2::bufread::GzDecoder;
use ini::Ini;
use rfd::{FileDialog, MessageButtons, MessageDialog};
use zip::ZipArchive;

pub fn download_wit(drive_mount_point: &Path) -> Result<()> {
    let file_name = if cfg!(target_os = "macos") {
        "wit-v3.04a-r8427-mac.tar.gz"
    } else if cfg!(target_os = "windows") {
        "wit-v3.04a-r8427-cygwin64.zip"
    } else {
        "wit-v3.04a-r8427-x86_64.tar.gz"
    };

    let download_url = format!("https://wit.wiimm.de/download/{file_name}");
    let resp = ureq::get(&download_url).call()?;

    if cfg!(target_family = "unix") {
        let reader = BufReader::new(resp.into_reader());
        let d = GzDecoder::new(reader);
        let mut a = tar::Archive::new(d);
        a.unpack(drive_mount_point)?;
    } else {
        let size = resp.header("Content-Length").unwrap().parse::<usize>()?;
        let mut reader = resp.into_reader();
        let mut cache = Vec::with_capacity(size);
        reader.read_to_end(&mut cache)?;

        let reader = Cursor::new(cache);
        let mut a = ZipArchive::new(reader)?;
        a.extract(drive_mount_point)?;
    }

    Ok(())
}

pub fn get_wit_path(drive_mount_point: &Path) -> Result<PathBuf> {
    let base_dir = if cfg!(target_os = "macos") {
        drive_mount_point.join("wit-v3.04a-r8427-mac")
    } else if cfg!(target_os = "windows") {
        drive_mount_point.join("wit-v3.04a-r8427-cygwin64")
    } else {
        drive_mount_point.join("wit-v3.04a-r8427-x86_64")
    };

    let wit_path = if cfg!(target_family = "unix") {
        base_dir.join("bin").join("wit")
    } else {
        base_dir.join("bin").join("wit.exe")
    };

    Ok(wit_path)
}

#[derive(Debug, Clone)]
pub struct Game {
    pub id: String,
    pub title: String,
    pub size: String,
    pub path: String,
}

pub fn get_games(drive_mount_point: &Path) -> Result<Vec<Game>> {
    let wbfs_folder = drive_mount_point.join("wbfs");
    if !wbfs_folder.exists() {
        fs::create_dir(&wbfs_folder)?;
    }

    let wit_path = get_wit_path(drive_mount_point)?;

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

    Ok(games)
}

pub fn select_games() -> Vec<PathBuf> {
    let games = FileDialog::new().add_filter("ISO", &["iso"]).pick_files();

    match games {
        Some(games) => games,
        None => Vec::new(),
    }
}

pub fn add_game(drive_mount_point: &Path, game_path: &Path) -> Result<()> {
    let wit_path = get_wit_path(drive_mount_point)?;

    let output = Command::new(&wit_path).arg("id6").arg(game_path).output()?;
    let game_id = String::from_utf8(output.stdout)?.trim().to_string();

    let path = drive_mount_point
        .join("wbfs")
        .join(game_id)
        .with_extension("wbfs");

    let output = Command::new(wit_path)
        .arg("copy")
        .arg(game_path)
        .arg(path)
        .arg("--split")
        .arg("--split-size")
        .arg("4G-32K")
        .output()?;
    println!("{:?}", output);

    Ok(())
}

pub fn remove_game(drive_mount_point: &Path, game: &Game) -> Result<()> {
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

    Ok(())
}
