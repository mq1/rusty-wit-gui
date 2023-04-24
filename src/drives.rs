// SPDX-FileCopyrightText: 2023 Manuel Quarneti <hi@mq1.eu>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt, path::PathBuf};

use anyhow::Result;
use sysinfo::{DiskExt, System, SystemExt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drive {
    pub name: String,
    pub total_space: String,
    pub available_space: String,
    pub mount_point: PathBuf,
}

impl fmt::Display for Drive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}/{} GiB)",
            self.name, self.available_space, self.total_space
        )
    }
}

pub fn list() -> Result<Vec<Drive>> {
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
            let mount_point = disk.mount_point();

            Drive {
                name: name.into(),
                total_space: total_space_gib.into(),
                available_space: available_space_gib.into(),
                mount_point: mount_point.to_path_buf(),
            }
        })
        .collect::<Vec<_>>();

    Ok(drives)
}
