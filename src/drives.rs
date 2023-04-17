use crate::Drive;

use std::{
    fs::{self, Permissions},
    os::unix::prelude::PermissionsExt,
    process::Command,
};

use anyhow::{anyhow, Result};
use rfd::{MessageButtons, MessageDialog};
use slint::{Model, ModelRc, VecModel};
use sysinfo::{DiskExt, System, SystemExt};
use tempfile::NamedTempFile;

const ASKPASS: &str = include_str!("../data/sudo-askpass.osascript-en.js");

pub fn list() -> Result<ModelRc<Drive>> {
    let mut sys = System::new();
    sys.refresh_disks_list();

    let drives = sys
        .disks()
        .iter()
        .filter(|disk| disk.is_removable())
        .map(|disk| {
            let name = disk.name().to_string_lossy().to_string();
            let total_space_gib = format!("{:.2}", disk.total_space() as f32 / 1073741824.);
            let available_space_gib = format!("{:.2}", disk.available_space() as f32 / 1073741824.);
            let mount_point = disk.mount_point().to_string_lossy().to_string();

            let path = if cfg!(target_os = "macos") {
                let output = Command::new("diskutil")
                    .arg("info")
                    .arg("-plist")
                    .arg(&mount_point)
                    .output()
                    .expect("Failed to execute diskutil");

                let output = plist::from_bytes::<plist::Value>(&output.stdout)
                    .expect("Failed to parse plist");

                let device_node = output
                    .as_dictionary()
                    .and_then(|dict| dict.get("DeviceNode"))
                    .and_then(|node| node.as_string());

                device_node.unwrap().to_string()
            } else {
                mount_point.clone()
            };

            Drive {
                name: name.into(),
                total_space: total_space_gib.into(),
                available_space: available_space_gib.into(),
                path: path.into(),
                mount_point: mount_point.into(),
            }
        })
        .collect::<Vec<_>>();

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
                .arg(drive.path.as_str())
                .output()?;
            println!("{:?}", output);

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
                .arg(drive.path.as_str())
                .env("SUDO_ASKPASS", file.path().to_string_lossy().to_string())
                .output()?;
            println!("{:?}", output);

            fs::remove_file(file.path())?;

            let output = Command::new("diskutil")
                .arg("mount")
                .arg(drive.path.as_str())
                .output()?;
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
