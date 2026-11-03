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

#![feature(iterator_try_collect)]

#[macro_use]
extern crate libtest_mimic_collect;

mod dispatcher;
mod display;
mod display_listener;
mod error;
mod event_loop;
mod keybinding_listener;
mod models;
mod mouse_listener;
mod native_window;
mod platform_event;
mod platform_impl;
mod single_instance;
mod thread_bound;
mod window_listener;

pub use dispatcher::*;
pub use display::*;
pub use display_listener::*;
pub use error::*;
pub use event_loop::*;
pub use keybinding_listener::*;
pub use models::*;
pub use mouse_listener::*;
pub use native_window::*;
pub use platform_event::*;
pub use single_instance::*;
pub use thread_bound::*;
pub use window_listener::*;

pub fn main() {
  // Due to macOS requiring the main thread for some UI APIs, these
  // tests must execute on the main thread. Until this is natively
  // supported via cargo's test harness, we use `libtest_mimic_collect`.
  //
  // To run these tests, run `cargo test <...args> -- --test-threads=1`.
  //
  // Ref: https://github.com/rust-lang/rust/issues/104053
  libtest_mimic_collect::TestCollection::run();
}
