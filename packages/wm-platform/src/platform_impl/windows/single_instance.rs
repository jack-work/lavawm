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

/// Arbitrary GUID to uniquely identify the application.
const APP_GUID: PCWSTR =
  w!("Global\\a1b2c3d4-5e6f-7a8b-9c0d-lavawm000001");

/// Platform-specific implementation of [`SingleInstance`].
pub struct SingleInstance {
  handle: HANDLE,
}

impl SingleInstance {
  /// Implements [`SingleInstance::new`].
  pub(crate) fn new() -> crate::Result<Self> {
    // Create a named system-wide mutex.
    let handle = unsafe { CreateMutexW(None, true, APP_GUID) }?;

    if let Err(err) = unsafe { GetLastError() } {
      if err == ERROR_ALREADY_EXISTS.into() {
        return Err(crate::Error::Platform(
          "Another instance of the application is already running."
            .to_string(),
        ));
      }
    }

    Ok(Self { handle })
  }

  /// Implements [`SingleInstance::is_running`].
  #[must_use]
  pub(crate) fn is_running() -> bool {
    let res = unsafe {
      OpenMutexW(SYNCHRONIZATION_ACCESS_RIGHTS::default(), false, APP_GUID)
    };

    // If the mutex exists, then another instance is running.
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
