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

use anyhow::Context;
use tracing::info;
use wm_platform::Platform;

use crate::{
  commands::monitor::{
    add_monitor, bind_workspaces_to_monitor, remove_monitor,
    sort_monitors, update_monitor,
  },
  models::Monitor,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_display_settings_changed(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!("Display settings changed.");

  let native_monitors = Platform::sorted_monitors()?;

  let hardware_ids = native_monitors
    .iter()
    .filter_map(|m| m.hardware_id().ok())
    .flatten()
    .cloned()
    .collect::<Vec<_>>();

  let mut pending_monitors = state.monitors();
  let mut new_native_monitors = Vec::new();

  for native_monitor in native_monitors {
    // Get the corresponding `Monitor` instance by either its handle,
    // device path, or hardware ID. Monitor handles and device paths
    // *should* be unique, but can both change over time. The hardware ID
    // is not guaranteed to be unique, so we match against that last.
    let found_monitor = pending_monitors
      .iter()
      .find_map(|monitor| {
        if monitor.native().handle == native_monitor.handle {
          return Some(monitor);
        }

        if monitor.native().device_path().ok()??
          == native_monitor.device_path().ok()??
        {
          return Some(monitor);
        }

        let hardware_id = monitor.native().hardware_id().ok()??.clone();

        // Check that there aren't multiple monitors with the same
        // hardware ID.
        let is_unique =
          hardware_ids.iter().filter(|&id| *id == hardware_id).count()
            == 1;

        if is_unique
          && hardware_id == *native_monitor.hardware_id().ok()??
        {
          Some(monitor)
        } else {
          None
        }
      })
      .cloned();

    match found_monitor {
      Some(found_monitor) => {
        // Remove the monitor from the pending list so that we don't
        // consider it again in the next iteration.
        if let Some(index) = pending_monitors
          .iter()
          .position(|m| m.id() == found_monitor.id())
        {
          pending_monitors.remove(index);
        }

        update_monitor(&found_monitor, native_monitor, state)?;
      }
      None => {
        new_native_monitors.push(native_monitor);
      }
    }
  }

  let mut newly_added_monitors: Vec<Monitor> = Vec::new();

  for native_monitor in new_native_monitors {
    match pending_monitors.first() {
      Some(_) => {
        let monitor = pending_monitors.remove(0);
        update_monitor(&monitor, native_monitor, state)?;
      }
      // Add monitor if it doesn't exist in state. Workspace binding
      // is deferred until after monitors are sorted.
      None => {
        let monitor = add_monitor(native_monitor, state)?;
        newly_added_monitors.push(monitor);
      }
    }
  }

  // Remove any monitors that no longer exist and move their workspaces
  // to other monitors.
  //
  // Prevent removal of the last monitor (i.e. for when all monitors are
  // disconnected). This will cause the WM's monitors to mismatch the OS
  // monitor state, however, it'll be updated correctly when a new monitor
  // is connected again.
  for pending_monitor in pending_monitors {
    if state.monitors().len() != 1 {
      remove_monitor(pending_monitor, state, config)?;
    }
  }

  // Sort monitors by position *before* binding workspaces, so that
  // `monitor.index()` reflects the correct sorted position.
  sort_monitors(&state.root_container)?;

  // Now bind workspaces to newly added monitors using correct indices.
  for monitor in newly_added_monitors {
    bind_workspaces_to_monitor(&monitor, state, config)?;
  }

  for window in state.windows() {
    // Display setting changes can spread windows out sporadically, so mark
    // all windows as needing a DPI adjustment (just in case).
    window.set_has_pending_dpi_adjustment(true);

    // Need to update floating position of moved windows when a monitor is
    // disconnected or if the primary display is changed. The primary
    // display dictates the position of 0,0.
    let workspace = window.workspace().context("No workspace.")?;

    let should_recenter = if window.has_custom_floating_placement() {
      let workspace_rect = workspace.to_rect()?;

      // Keep the placement if it still intersects the workspace, since
      // `PlatformEvent::DisplaySettingsChanged` can be triggered by
      // non-monitor changes (e.g. unplugging a USB device).
      !window.floating_placement().has_overlap_x(&workspace_rect)
        || !window.floating_placement().has_overlap_y(&workspace_rect)
    } else {
      true
    };

    if should_recenter {
      window.set_floating_placement(
        window
          .floating_placement()
          .translate_to_center(&workspace.to_rect()?),
      );
    }
  }

  // Redraw full container tree.
  state
    .pending_sync
    .queue_container_to_redraw(state.root_container.clone());

  Ok(())
}
