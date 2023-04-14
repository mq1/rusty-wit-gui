// SPDX-FileCopyrightText: 2023 Manuel Quarneti <hi@mq1.eu>
// SPDX-License-Identifier: GPL-3.0-only

fn main() {
    let config =
        slint_build::CompilerConfiguration::new()
        .with_style("material".into());

    slint_build::compile_with_config("ui/appwindow.slint", config).unwrap();
}
