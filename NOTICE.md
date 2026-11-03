# Notice of Modifications

LavaWM is a modified version of [GlazeWM](https://github.com/glzr-io/glazewm),
Copyright (C) 2024 glzr-io, licensed under the GNU General Public License
v3.0. LavaWM is distributed under the same license; see [LICENSE.md](./LICENSE.md).

As required by section 5(a) of the GPL-3.0, this file records how LavaWM
differs from GlazeWM and when. Modified Rust sources also carry the dual
copyright header described in [CLAUDE.md](./CLAUDE.md).

## Upstream base

| Field | Value |
|---|---|
| Upstream | `glzr-io/glazewm`, branch `main` |
| Base commit | `5709ad0a3c7c386bbc3e38166a865ffc12937515` (2026-04-09) |
| Latest upstream release included | `v3.10.1` |
| Last synced | 2026-11-03 (upstream `main` had no newer commits) |

## Modifications

### 2026-02-10 to 2026-04-29 (LavaWM 0.1.0 to 0.3.1)

- Renamed binaries (`lavawm`, `lavawm-cli`, `lavawm-watcher`), config
  directory (`~/.glzr/lavawm/`), `LAVAWM_CONFIG_PATH`, and single-instance
  mutex so LavaWM can be installed beside GlazeWM. The IPC port stays 6123
  for Zebar.
- Added `wm-cleanup-windows` (remove windows whose handles are invalid),
  `wm-inject-ghost` (debug), and `wm-recover-windows` (move every window
  from hidden workspaces, plus unmanaged visible windows, onto the focused
  workspace).
- Recover windows left cloaked by a previous WM instance at startup.
- Per-window cooldown on fullscreen transitions to stop RDP and similar
  clients oscillating between fullscreen and floating.
- Fixed `toggle-fullscreen` not exiting fullscreen and corrupting tiling
  direction.
- Deactivate empty workspaces when their monitor is removed.
- Rebranded README, icon, and tray; replaced CI with LavaWM lint and release
  pipelines and a Scoop bucket; added GPL headers and `CLAUDE.md`.

### 2026-11-03

- Restored the LavaWM single-instance mutex, error log path
  (`~/.glzr/lavawm/errors.log`), tray tooltip, and run-on-startup name,
  which the April rebase onto upstream had reverted to GlazeWM's.
- Windows: display changes received while suspended are no longer lost; a
  catch-up reconciliation runs five seconds after wake, matching macOS.
- Added `general.display_change_commands`, run once the display layout
  settles after a monitor change or wake.
- `shell-exec` no longer leaks a process handle per invocation.
- Fullscreen cooldowns are keyed by `WindowId` and pruned; `wm` builds on
  macOS again.
- Minimizing, floating or fullscreening a window that was the only child
  of nested split containers no longer leaves an empty split holding a
  blank tiling slot. Empty split children are removed whenever split
  containers are flattened.
- CI runs the `wm` unit tests; lint fixes so CI passes; `wm-watcher` checks the target OS rather than
  the host; shell scripts are checked out with LF line endings.
