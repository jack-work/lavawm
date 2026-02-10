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

use crate::platform_impl;

/// Ensures only one instance of the application is running at a time.
///
/// # Platform-specific
///
/// - **Windows**: Uses a named system-wide mutex.
/// - **macOS**: Uses an exclusive file lock.
pub struct SingleInstance {
  /// Inner platform-specific single instance implementation.
  _inner: platform_impl::SingleInstance,
}

impl SingleInstance {
  /// Creates a new [`SingleInstance`], acquiring the platform-specific
  /// lock or mutex.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Platform`] if another instance is already running.
  pub fn new() -> crate::Result<Self> {
    let inner = platform_impl::SingleInstance::new()?;
    Ok(Self { _inner: inner })
  }

  /// Returns whether another instance of the application is currently
  /// running.
  #[must_use]
  pub fn is_running() -> bool {
    platform_impl::SingleInstance::is_running()
  }
}
