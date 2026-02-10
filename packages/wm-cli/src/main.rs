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

use std::{env, process::Command};

use anyhow::Context;
use wm_cli::start;
use wm_common::AppCommand;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = std::env::args().collect::<Vec<_>>();
  let app_command = AppCommand::parse_with_default(&args);

  match app_command {
    AppCommand::Start { .. } => {
      let exe_path = env::current_exe()?;
      let exe_dir = exe_path
        .parent()
        .context("Failed to resolve path to the current executable.")?
        .to_owned();

      // Main executable is either in the current directory (when running
      // debug/release builds) or in the parent directory when packaged.
      let main_path =
        [exe_dir.join("lavawm.exe"), exe_dir.join("../lavawm.exe")]
          .into_iter()
          .find(|path| path.exists() && *path != exe_path)
          .and_then(|path| path.to_str().map(ToString::to_string))
          .context("Failed to resolve path to the main executable.")?;

      // UIAccess applications can't be started directly, so we need to use
      // CMD to start it. The start command is used to avoid a long-running
      // CMD process in the background.
      Command::new("cmd")
        .args(
          ["/C", "start", "", &main_path]
            .into_iter()
            .chain(args.iter().skip(1).map(String::as_str)),
        )
        .spawn()
        .context("Failed to start main executable.")?;

      Ok(())
    }
    _ => start(args).await,
  }
}
