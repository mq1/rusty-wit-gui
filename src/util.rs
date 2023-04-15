// SPDX-FileCopyrightText: 2023 Manuel Quarneti <hi@mq1.eu>
// SPDX-License-Identifier: GPL-3.0-only

use crate::Drive;

use std::{
    fs::{self, Permissions},
    os::unix::prelude::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, Result};
use regex::Regex;
use rfd::{FileDialog, MessageButtons, MessageDialog};
use slint::{ModelRc, SharedString, VecModel};
use sysinfo::{DiskExt, System, SystemExt};
use tempfile::NamedTempFile;

const ASKPASS: &str = include_str!("../data/sudo-askpass.osascript-en.js");

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

pub fn refresh_drive(drive: Drive) -> Result<Drive> {
    let mut sys = System::new();
    sys.refresh_disks_list();

    let disk = sys
        .disks()
        .iter()
        .find(|disk| disk.mount_point().to_string_lossy().to_string() == drive.path.to_string())
        .ok_or(anyhow!("Drive not found"))?;

    let total_space_gib = disk.total_space() as f32 / 1024. / 1024. / 1024.;
    let available_space_gib = disk.available_space() as f32 / 1024. / 1024. / 1024.;

    Ok(Drive {
        name: drive.name,
        total_space: format!("{:.2} GiB", total_space_gib).into(),
        available_space: format!("{:.2} GiB", available_space_gib).into(),
        path: drive.path,
    })
}

pub fn format_drive(drive: &Drive) -> Result<()> {
    let yes = MessageDialog::new()
        .set_title("Format drive")
        .set_description(&format!(
            "Are you sure you want to format {} ({})?\nThis will erase all data on the drive.\nThe drive will be formatted as FAT32 with the name WII.",
            drive.name, drive.path
        ))
        .set_buttons(MessageButtons::OkCancel)
        .show();

    if yes {
        if cfg!(target_os = "macos") {
            let output = Command::new("diskutil")
                .arg("umount")
                .arg(drive.path.to_string())
                .output()?;
            println!("{:?}", output);

            let stdout = String::from_utf8_lossy(&output.stdout);
            let re = Regex::new("Volume .+ on (.+) unmounted")?;
            let caps = re
                .captures(&stdout)
                .ok_or(anyhow!("Could not parse diskutil output"))?;
            let disk = caps
                .get(1)
                .ok_or(anyhow!("Could not parse diskutil output"))?
                .as_str();

            // This is a hack to get sudo to work on macOS
            let file = NamedTempFile::new()?;
            fs::write(file.path(), ASKPASS)?;
            let permissions = Permissions::from_mode(0o700);
            fs::set_permissions(file.path(), permissions)?;

            let output = Command::new("sudo")
                .arg("--askpass")
                .arg("newfs_msdos")
                .arg("-F")
                .arg("32")
                .arg("-b")
                .arg("32768")
                .arg("-v")
                .arg("WII")
                .arg(format!("/dev/{disk}"))
                .env("SUDO_ASKPASS", file.path().to_string_lossy().to_string())
                .output()?;
            println!("{:?}", output);

            fs::remove_file(file.path())?;

            let output = Command::new("diskutil").arg("mount").arg(disk).output()?;
            println!("{:?}", output);
        } else {
            MessageDialog::new()
                .set_title("Not implemented")
                .set_description("This feature is only available on macOS")
                .show();
        }
    }

    Ok(())
}

pub fn get_games(drive_path: &str) -> Result<ModelRc<SharedString>> {
    let wbfs_folder = Path::new(drive_path).join("wbfs");
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

fn get_titles(drive_path: &str) -> Result<()> {
    let titles_path = Path::new(drive_path).join("titles.txt");

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

pub fn add_game(drive_path: &str, game: &Path) -> Result<()> {
    let wbfs_folder = Path::new(drive_path).join("wbfs");
    let titles_path = Path::new(drive_path).join("titles.txt");

    get_titles(drive_path)?;

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

pub fn remove_game(drive_path: &str, game: &str) -> Result<ModelRc<SharedString>> {
    let yes = MessageDialog::new()
        .set_title("Remove Game")
        .set_description(&format!("Are you sure you want to remove {game}?"))
        .set_buttons(MessageButtons::OkCancel)
        .show();

    if yes {
        let wbfs_folder = Path::new(drive_path).join("wbfs");
        let game_path = wbfs_folder.join(game);

        fs::remove_file(game_path)?;
    }

    get_games(drive_path)
}
