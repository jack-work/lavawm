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

  // When the `ui_access` feature is enabled, the `uiAccess` attribute is
  // set to `true`. UIAccess is disabled by default because it requires the
  // application to be signed and installed in a secure location.
  let ui_access = {
    #[cfg(feature = "ui_access")]
    {
      "true"
    }
    #[cfg(not(feature = "ui_access"))]
    {
      "false"
    }
  };

  // Conditionally enable UIAccess, which grants privilege to set the
  // foreground window and to set the position of elevated windows.
  //
  // Ref: https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/security-policy-settings/user-account-control-only-elevate-uiaccess-applications-that-are-installed-in-secure-locations
  //
  // Additionally, declare support for per-monitor DPI awareness.
  let manifest_str = format!(
    r#"
<assembly
  xmlns="urn:schemas-microsoft-com:asm.v1"
  manifestVersion="1.0"
  xmlns:asmv3="urn:schemas-microsoft-com:asm.v3"
>
  <asmv3:trustInfo>
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="{ui_access}" />
      </requestedPrivileges>
    </security>
  </asmv3:trustInfo>

  <asmv3:application>
    <windowsSettings
      xmlns:ws2005="http://schemas.microsoft.com/SMI/2005/WindowsSettings"
      xmlns:ws2016="http://schemas.microsoft.com/SMI/2016/WindowsSettings"
    >
      <ws2005:dpiAware>true</ws2005:dpiAware>
      <ws2016:dpiAwareness>PerMonitorV2</ws2016:dpiAwareness>
    </windowsSettings>
  </asmv3:application>
</assembly>
"#
  );

  res.set_manifest(&manifest_str);
  res.set_icon("../../resources/assets/icon.ico");

  // Set language to English (US).
  res.set_language(0x0409);

  res.set("OriginalFilename", "lavawm.exe");
  res.set("ProductName", "LavaWM");
  res.set("FileDescription", "LavaWM");

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
