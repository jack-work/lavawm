<div align="center">
  <br>
  <table><tr>
    <td><img src="./lavalogo.png" width="200" alt="LavaWM logo" /></td>
    <td>
      <h1 style="font-family: 'Palatino Linotype', 'Book Antiqua', Palatino, Georgia, 'Times New Roman', serif; font-size: 4em; margin: 0; letter-spacing: 0.08em;">𝐋 𝐀 𝐕 𝐀</h1>
      <p><em>a tiling window manager,<br>layered with care</em></p>
    </td>
  </tr></table>

**Fork of [GlazeWM](https://github.com/glzr-io/glazewm) &mdash; built for Windows on ARM, shaped by baklava and ambition.**

</div>

---

## What is this?

**LavaWM** is a keyboard-driven tiling window manager for Windows, forked from the excellent [GlazeWM v3](https://github.com/glzr-io/glazewm). It inherits all the golden, flaky layers of GlazeWM and adds a few of its own:

- Ghost window cleanup (`wm-cleanup-windows`) &mdash; manually sweep away stale windows
- Window recovery (`wm-recover-windows`) &mdash; pull every window back onto the focused workspace
- Fullscreen oscillation fix &mdash; stops RDP and other apps from fighting the WM in a rapid fullscreen/floating loop
- Dock and sleep resilience &mdash; reconciles monitors after wake and runs `display_change_commands` (e.g. restart Zebar) once the layout settles
- Windows on ARM (aarch64) first-class build support
- Can run side-by-side with a stock GlazeWM installation

### Key features (inherited from GlazeWM)

- Simple YAML configuration
- Multi-monitor support
- Customizable rules for specific windows
- Integration with [Zebar](https://github.com/glzr-io/zebar) as a status bar
- Binding modes, window effects, and more

## Full Disclosure

This project is managed largely by [Claude Code](https://claude.com/claude-code) (Anthropic's CLI agent). A human ([@jack-work](https://github.com/jack-work)) steers the ship, reviews the output, and eats the baklava &mdash; but the bulk of the code changes, architecture exploration, bug investigation, and even this README were produced by an AI pair-programming session.

**The author has so much baklava he needs help eating it all.** That means:

- PRs are welcome and appreciated
- If you want to understand how something works, open an issue &mdash; teaching is half the fun
- If a dependency develops or something breaks, extra hands make light work
- If the WM does something weird, it might be the AI's fault. File a bug.

## Getting Started

### Install via Scoop

```powershell
scoop bucket add lavawm https://github.com/jack-work/scoop-lavawm
scoop install lavawm
```

### Download from GitHub Releases

Grab the latest zip for your architecture from [Releases](https://github.com/jack-work/lavawm/releases):
- **x64**: `lavawm-vX.Y.Z-x64.zip`
- **ARM64**: `lavawm-vX.Y.Z-arm64.zip`

### Build from source

You'll need the Rust toolchain (nightly recommended for ARM targets):

```sh
git clone https://github.com/jack-work/lavawm.git
cd lavawm
cargo build --release
```

Binaries land in `target/release/`:
- `lavawm.exe` &mdash; the window manager
- `lavawm-cli.exe` &mdash; CLI client for commands/queries
- `lavawm-watcher.exe` &mdash; auto-restart watcher

### Run it

```sh
# Start the WM (generates default config on first run)
lavawm start

# Or with a custom config
lavawm start --config="C:\path\to\config.yaml"
```

Config lives at `%userprofile%\.glzr\lavawm\config.yaml` by default, or set `LAVAWM_CONFIG_PATH`.

### Coexistence with GlazeWM

LavaWM uses a separate config directory (`~/.glzr/lavawm/`), a separate single-instance mutex, and renamed binaries. You can have both installed &mdash; just don't run them at the same time (they share IPC port 6123 for Zebar compatibility).

## Default Keybindings

On first launch, a default config is generated. Here's the cheat sheet:

![Keybinding cheatsheet](/resources/assets/cheatsheet.png)

## Fork It

Seriously &mdash; **fork this repo and try modifying it yourself.** The codebase is well-structured (thanks to the upstream GlazeWM team), and Claude Code can help you navigate it. Some ideas:

- Add your own WM commands
- Tweak the tiling algorithm
- Build integrations with your favorite tools
- Fix that one window that never tiles right

If you make something cool, changes are always appreciated upstream (to me). Open a PR at [jack-work/lavawm](https://github.com/jack-work/lavawm) and let's make it better together.

## Configuration

LavaWM uses the same YAML config format as GlazeWM v3. See the [upstream config documentation](https://github.com/glzr-io/glazewm#config-documentation) for the full reference, or check the [sample config](./resources/assets/sample-config.yaml).

Quick reference of LavaWM-specific commands:

| Command | Description |
|---------|-------------|
| `wm-cleanup-windows` | Remove ghost windows (invalid HWNDs) from the tree |
| `wm-recover-windows` | Move all windows from hidden workspaces, and unmanaged visible windows, to the focused workspace |
| `wm-inject-ghost` | Debug: inject a fake ghost for testing cleanup |

LavaWM-specific config:

| Key | Description |
|---------|-------------|
| `general.display_change_commands` | Commands run 3s after the last monitor connect, disconnect, rearrange or rescale, or after waking from sleep. Working-area changes alone do not trigger them. |

## Zebar after docking or sleep

Zebar 3.x can lose its bars when the display layout changes: its relaunch drops every widget from its own state before recreating them, so one failed or stalled recreation leaves bars stranded at old monitor coordinates (or unpositioned behind other windows), with no tracked widgets and nothing left to relaunch. A second `zebar` or `zebar startup` is forwarded to the wedged instance and does nothing, so only a process restart recovers it.

LavaWM restarts it for you once the layout settles:

```yaml
general:
  display_change_commands: ['shell-exec --hide-window powershell -NoProfile -Command "Stop-Process -Name zebar -Force -ErrorAction Ignore; Wait-Process -Name zebar -Timeout 5 -ErrorAction Ignore; Start-Process zebar"']
```

Bind the same command to a key for manual recovery. The stop and start run in one process so the new instance never races the old one's single-instance lock. If LavaWM runs elevated, Zebar does too, and only an elevated process (such as LavaWM's `shell-exec`) can stop it.

## Known Issues

- **Fullscreen &rarr; minimize loop**: Some windows (especially RDP clients) can oscillate between fullscreen and floating states. A per-window cooldown mitigates this, but edge cases remain. See upstream issues [#1070](https://github.com/glzr-io/glazewm/issues/1070), [#906](https://github.com/glzr-io/glazewm/issues/906).
- **Toggle fullscreen sometimes minimizes**: If a window was minimized before going fullscreen, `toggle-fullscreen` may not restore correctly. The upstream fix (PR [#1071](https://github.com/glzr-io/glazewm/pull/1071)) is included, but some edge cases persist.

## License

This project is licensed under **GPL-3.0**, same as the upstream GlazeWM. See [LICENSE.md](./LICENSE.md) for details, and [NOTICE.md](./NOTICE.md) for the upstream base and a dated record of what LavaWM changed.

You are free to fork, modify, and redistribute &mdash; but derivative works must also be GPL-3.0 and source must be made available.

## Acknowledgments

- [GlazeWM](https://github.com/glzr-io/glazewm) by [glzr-io](https://github.com/glzr-io) &mdash; the foundation this is built on
- [i3wm](https://i3wm.org/) &mdash; the original inspiration
- [Zebar](https://github.com/glzr-io/zebar) &mdash; the status bar that ties it all together
- [Claude Code](https://claude.com/claude-code) &mdash; the AI that wrote most of this

---

<div align="center">

*Baked with pistachios and honey by [@jack-work](https://github.com/jack-work)*

*𝐋𝐀𝐕𝐀 &mdash; layer by layer*

</div>
