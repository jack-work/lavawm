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

use anyhow::{bail, Context, Result};
use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{
      CloseHandle, GetLastError, ERROR_ALREADY_EXISTS,
      ERROR_FILE_NOT_FOUND, HANDLE,
    },
    System::Threading::{
      CreateMutexW, OpenMutexW, ReleaseMutex,
      SYNCHRONIZATION_ACCESS_RIGHTS,
    },
  },
};

pub struct SingleInstance {
  handle: HANDLE,
}

/// Arbitrary GUID used to identify the application.
/// Changed from GlazeWM's GUID to allow side-by-side operation.
const APP_GUID: PCWSTR =
  w!("Global\\a1b2c3d4-5e6f-7a8b-9c0d-lavawm000001");

impl SingleInstance {
  /// Creates a new system-wide mutex to ensure that only one instance of
  /// the application is running.
  pub fn new() -> Result<Self> {
    let handle = unsafe { CreateMutexW(None, true, APP_GUID) }
      .context("Failed to create single instance mutex.")?;

    if let Err(err) = unsafe { GetLastError() } {
      if err == ERROR_ALREADY_EXISTS.into() {
        bail!("Another instance of the application is already running.");
      }
    }

    Ok(Self { handle })
  }

  /// Gets whether there is an active instance of the application.
  #[must_use]
  pub fn is_running() -> bool {
    let res = unsafe {
      OpenMutexW(SYNCHRONIZATION_ACCESS_RIGHTS::default(), false, APP_GUID)
    };

    // Check whether the mutex exists. If it doesn't, then this is the
    // only instance.
    match res {
      Ok(_) => false,
      Err(err) => err == ERROR_FILE_NOT_FOUND.into(),
    }
  }
}

impl Drop for SingleInstance {
  fn drop(&mut self) {
    unsafe {
      let _ = ReleaseMutex(self.handle);
      let _ = CloseHandle(self.handle);
    }
  }
}
