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

use tokio::sync::mpsc;

use crate::{platform_impl, Dispatcher};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayEvent {
  SettingsChanged,
  Resumed,
}

/// A listener for system-wide display setting changes.
///
/// Detects changes to display configuration including resolution changes,
/// display connections/disconnections, and working area changes.
pub struct DisplayListener {
  event_rx: mpsc::UnboundedReceiver<DisplayEvent>,

  /// Inner platform-specific display listener.
  inner: platform_impl::DisplayListener,
}

impl DisplayListener {
  /// Creates a new [`DisplayListener`].
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let inner = platform_impl::DisplayListener::new(event_tx, dispatcher)?;
    Ok(Self { event_rx, inner })
  }

  /// Returns when the next display settings change is detected.
  ///
  /// Returns `None` if the channel has been closed.
  pub async fn next_event(&mut self) -> Option<DisplayEvent> {
    self.event_rx.recv().await
  }

  /// Terminates the display listener.
  pub fn terminate(&mut self) -> crate::Result<()> {
    self.inner.terminate()
  }
}
