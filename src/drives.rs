use crate::Drive;

use std::{
    fs::{self, Permissions},
    os::unix::prelude::PermissionsExt,
    path::PathBuf,
    process::Command,
};

use anyhow::{anyhow, Result};
use regex::Regex;
use rfd::{MessageButtons, MessageDialog};
use serde::Deserialize;
use slint::{Model, ModelRc, VecModel};
use tempfile::NamedTempFile;

const ASKPASS: &str = include_str!("../data/sudo-askpass.osascript-en.js");

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DiskutilList {
    all_disks_and_partitions: Vec<DiskutilDisk>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DiskutilDisk {
    partitions: Vec<DiskutilPartition>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DiskutilPartition {
    device_identifier: String,
    mount_point: String,
    volume_name: String,
    size: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DiskutilInfo {
    free_space: u64,
}

pub fn list() -> Result<ModelRc<Drive>> {
    let mut drives = Vec::new();

    if cfg!(target_os = "macos") {
        let output = Command::new("diskutil")
            .arg("list")
            .arg("-plist")
            .arg("external")
            .arg("physical")
            .output()?;

        let diskutil_list: DiskutilList = plist::from_bytes(&output.stdout)?;

        for disk in diskutil_list.all_disks_and_partitions {
            for partition in disk.partitions {
                let output = Command::new("diskutil")
                    .arg("info")
                    .arg("-plist")
                    .arg(&partition.device_identifier)
                    .output()?;

                let diskutil_info: DiskutilInfo = plist::from_bytes(&output.stdout)?;

                let total_space = format!("{:.2}", partition.size as f32 / 1073741824.);
                let free_space = format!("{:.2}", diskutil_info.free_space as f32 / 1073741824.);

                drives.push(Drive {
                    name: partition.volume_name.into(),
                    total_space: total_space.into(),
                    available_space: free_space.into(),
                    path: PathBuf::from(format!("/dev/{}", partition.device_identifier))
                        .to_string_lossy()
                        .to_string()
                        .into(),
                    mount_point: partition.mount_point.into(),
                });
            }
        }
    }

    Ok(VecModel::from_slice(&drives))
}

pub fn refresh(drive: Drive) -> Result<Drive> {
    let drive = list()?
        .iter()
        .find(|d| d.path == drive.path)
        .ok_or(anyhow!("Drive not found"))?
        .clone();

    Ok(drive)
}

pub fn format(drive: &Drive) -> Result<()> {
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
