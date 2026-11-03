# Claude Code Instructions for LavaWM

## GPL-3.0 Compliance

When modifying or creating any `.rs` source file, ensure it has the GPL-3.0 copyright header at the top of the file. If the header is missing, add it before any other content:

```rust
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
```

Use `/gpl-notice` to batch-check and fix all modified files.

Record every user-visible change, and every upstream sync (with the new base commit), in [NOTICE.md](./NOTICE.md).

## Build

- Toolchain: Rust nightly, target `aarch64-pc-windows-msvc` (local) or both x64+arm64 (CI)
- Build: `cargo build --release`
- Binaries: `lavawm.exe`, `lavawm-cli.exe`, `lavawm-watcher.exe`
- Check from Linux or macOS: `cargo clippy --workspace --all-targets --all-features --target aarch64-pc-windows-msvc -- -D warnings`

## Branch

- Default branch is `master`
- CI runs lint on push to master and PRs
- Release CI triggers on tag push (`v*`)

## Naming

- Binary names: `lavawm`, `lavawm-cli`, `lavawm-watcher`
- Config dir and error log: `~/.glzr/lavawm/`
- Single-instance mutex: `Global\a1b2c3d4-5e6f-7a8b-9c0d-lavawm000001`
- Env var: `LAVAWM_CONFIG_PATH`
- IPC port: 6123 (shared with GlazeWM/Zebar)
- After an upstream rebase, grep for `glazewm`/`GlazeWM` in non-comment code: the April 2026 rebase silently reverted the mutex, log path and tray names
