// Copyright (C) 2024 glzr-io <https://github.com/glzr-io>
// Copyright (C) 2026 jack-work <https://github.com/jack-work>
//
// This file is part of LavaWM, a fork of GlazeWM.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use tauri_winres::VersionInfo;

fn main() {
  println!("cargo:rerun-if-env-changed=VERSION_NUMBER");
  let mut res = tauri_winres::WindowsResource::new();

  res.set_icon("../../resources/assets/icon.ico");

  // Set language to English (US).
  res.set_language(0x0409);

  res.set("OriginalFilename", "lavawm-cli.exe");
  res.set("ProductName", "LavaWM CLI");
  res.set("FileDescription", "LavaWM CLI");

  let version_parts = env!("VERSION_NUMBER")
    .split('.')
    .take(3)
    .map(|part| part.parse().unwrap_or(0))
    .collect::<Vec<u16>>();

  let [major, minor, patch] =
    <[u16; 3]>::try_from(version_parts).unwrap_or([0, 0, 0]);

  let version_str = format!("{major}.{minor}.{patch}.0");
  res.set("FileVersion", &version_str);
  res.set("ProductVersion", &version_str);

  let version_u64 = (u64::from(major) << 48)
    | (u64::from(minor) << 32)
    | (u64::from(patch) << 16);

  res.set_version_info(VersionInfo::FILEVERSION, version_u64);
  res.set_version_info(VersionInfo::PRODUCTVERSION, version_u64);

  res.compile().unwrap();
}
