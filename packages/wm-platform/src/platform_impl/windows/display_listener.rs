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
  sync::{Mutex, PoisonError},
  time::{Duration, Instant},
};

use tokio::sync::mpsc;
use tracing::warn;
use windows::Win32::UI::WindowsAndMessaging::{
  DBT_DEVNODES_CHANGED, PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND,
  PBT_APMSUSPEND, SPI_SETWORKAREA, WM_DEVICECHANGE, WM_DISPLAYCHANGE,
  WM_POWERBROADCAST, WM_SETTINGCHANGE,
};

use crate::{Dispatcher, DispatcherExtWindows, DisplayEvent};

const WAKE_COALESCE_DURATION: Duration = Duration::from_secs(5);

enum PowerState {
  Awake,
  Suspended,
  Waking(Instant),
}

impl PowerState {
  fn is_settling(&mut self) -> bool {
    match self {
      Self::Awake => false,
      Self::Suspended => true,
      Self::Waking(wake_time) => {
        let is_settling = wake_time.elapsed() < WAKE_COALESCE_DURATION;

        if !is_settling {
          *self = Self::Awake;
        }

        is_settling
      }
    }
  }
}

/// Platform-specific implementation of [`DisplayListener`].
pub(crate) struct DisplayListener {
  callback_id: Option<usize>,
  dispatcher: Dispatcher,
}

impl DisplayListener {
  /// Implements [`DisplayListener::new`].
  pub(crate) fn new(
    event_tx: mpsc::UnboundedSender<DisplayEvent>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let power_state = Mutex::new(PowerState::Awake);

    let callback_id = dispatcher.register_wndproc_callback(Box::new(
      move |_hwnd, message, wparam, _lparam| {
        let mut power_state =
          power_state.lock().unwrap_or_else(PoisonError::into_inner);

        match message {
          WM_POWERBROADCAST => {
            #[allow(clippy::cast_possible_truncation)]
            match wparam as u32 {
              // System is resuming from sleep/hibernation.
              PBT_APMRESUMEAUTOMATIC | PBT_APMRESUMESUSPEND => {
                let is_duplicate = matches!(
                  *power_state,
                  PowerState::Waking(wake_time)
                    if wake_time.elapsed() < WAKE_COALESCE_DURATION
                );

                if !is_duplicate {
                  *power_state = PowerState::Waking(Instant::now());

                  let event_tx = event_tx.clone();
                  std::thread::spawn(move || {
                    std::thread::sleep(WAKE_COALESCE_DURATION);
                    let _ = event_tx.send(DisplayEvent::Resumed);
                  });
                }
              }
              // System is entering sleep/hibernation.
              PBT_APMSUSPEND => {
                *power_state = PowerState::Suspended;
              }
              _ => {}
            }

            Some(0)
          }
          WM_DISPLAYCHANGE | WM_SETTINGCHANGE | WM_DEVICECHANGE => {
            let should_emit = {
              // Ignore display change messages if the system hasn't fully
              // resumed from sleep.
              if power_state.is_settling() {
                false
              } else {
                #[allow(clippy::cast_possible_truncation)]
                match message {
                  // Received when displays are connected and disconnected,
                  // resolution changes, or arrangement changes.
                  WM_DISPLAYCHANGE => true,
                  // Received when the working area has changed. Fires when
                  // the Windows taskbar is changed or an appbar is
                  // registered or changed. 3rd-party apps like
                  // ButteryTaskbar can trigger this message by calling
                  // `SystemParametersInfo(SPI_SETWORKAREA, ...)`.
                  WM_SETTINGCHANGE => wparam as u32 == SPI_SETWORKAREA.0,
                  // Received when any device is connected or disconnected
                  // (including non-display devices).
                  // TODO: Check if this is actually needed. Previous C#
                  // implementation did not use this.
                  WM_DEVICECHANGE => wparam as u32 == DBT_DEVNODES_CHANGED,
                  _ => unreachable!(),
                }
              }
            };

            if should_emit {
              let _ = event_tx.send(DisplayEvent::SettingsChanged);
            }

            Some(0)
          }
          _ => None,
        }
      },
    ))?;

    Ok(Self {
      callback_id: Some(callback_id),
      dispatcher: dispatcher.clone(),
    })
  }

  /// Implements [`DisplayListener::terminate`].
  pub(crate) fn terminate(&mut self) -> crate::Result<()> {
    if let Some(id) = self.callback_id.take() {
      self.dispatcher.deregister_wndproc_callback(id)?;
    }

    Ok(())
  }
}

impl Drop for DisplayListener {
  fn drop(&mut self) {
    if let Err(err) = self.terminate() {
      warn!("Failed to terminate display listener: {}", err);
    }
  }
}
