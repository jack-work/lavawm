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

use std::{
  fs::{self, File, TryLockError},
  path::PathBuf,
};

/// Platform-specific implementation of [`SingleInstance`].
pub(crate) struct SingleInstance {
  /// File that holds the lock.
  ///
  /// The lock is automatically released when the [`File`] is dropped.
  _file: File,
}

impl SingleInstance {
  /// Implements [`SingleInstance::new`].
  pub(crate) fn new() -> crate::Result<Self> {
    let path = Self::lock_file_path()?;

    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent).map_err(crate::Error::Io)?;
    }

    let file = File::create(&path).map_err(crate::Error::Io)?;

    // Acquire exclusive file lock.
    file.try_lock().map_err(|err| match err {
      TryLockError::WouldBlock => crate::Error::Platform(
        "Another instance of the application is already running."
          .to_string(),
      ),
      TryLockError::Error(io_err) => crate::Error::Io(io_err),
    })?;

    Ok(Self { _file: file })
  }

  /// Implements [`SingleInstance::is_running`].
  #[must_use]
  pub(crate) fn is_running() -> bool {
    let Ok(file) = Self::lock_file_path()
      .and_then(|path| File::open(&path).map_err(crate::Error::Io))
    else {
      return false;
    };

    // If `try_lock` fails with `WouldBlock`, the lock is held by another
    // process.
    file
      .try_lock()
      .is_err_and(|err| matches!(err, TryLockError::WouldBlock))
  }

  /// Returns the path to the lock file:
  /// `~/Library/Application Support/lavawm/.lock`.
  fn lock_file_path() -> crate::Result<PathBuf> {
    let home = home::home_dir().ok_or_else(|| {
      crate::Error::Platform(
        "Unable to determine home directory.".to_string(),
      )
    })?;

    Ok(home.join("Library/Application Support/lavawm/.lock"))
  }
}
