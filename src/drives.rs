// SPDX-FileCopyrightText: 2023 Manuel Quarneti <hi@mq1.eu>
// SPDX-License-Identifier: GPL-3.0-only

use crate::Drive;

use anyhow::{anyhow, Result};
use slint::{Model, ModelRc, VecModel};
use sysinfo::{DiskExt, System, SystemExt};

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

            Drive {
                name: name.into(),
                total_space: total_space_gib.into(),
                available_space: available_space_gib.into(),
                mount_point: mount_point.into(),
            }
        })
        .collect::<Vec<_>>();

    Ok(VecModel::from_slice(&drives))
}

pub fn refresh(drive: Drive) -> Result<Drive> {
    let drive = list()?
        .iter()
        .find(|d| d.mount_point == drive.mount_point)
        .ok_or_else(|| anyhow!("Drive not found"))?;

    Ok(drive)
}
