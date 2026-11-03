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

use super::{detach_container, flatten_split_container};
use crate::{
  models::Container,
  traits::{CommonGetters, TilingDirectionGetters},
};

/// Flattens any redundant split containers at the top-level of the given
/// parent container.
///
/// For example:
/// ```ignore,compile_fail
/// H[1 H[V[2, 3]]] -> H[1, 2, 3]
/// H[1 H[2, 3]] -> H[1, 2, 3]
/// H[V[1]] -> V[1]
/// ```
pub fn flatten_child_split_containers(
  parent: &Container,
) -> anyhow::Result<()> {
  let empty_split_children = parent
    .children()
    .into_iter()
    .filter(|child| child.is_split() && child.child_count() == 0)
    .collect::<Vec<_>>();

  for empty_split in empty_split_children {
    detach_container(empty_split)?;
  }

  if let Ok(parent) = parent.as_direction_container() {
    // Get children that are either tiling windows or split containers.
    let tiling_children = parent
      .children()
      .into_iter()
      .filter(|child| child.is_tiling_window() || child.is_split())
      .collect::<Vec<_>>();

    if tiling_children.len() == 1 {
      // Handle case where the parent is a split container and has a
      // single split container child.
      if let Some(split_child) = tiling_children[0].as_split() {
        flatten_split_container(split_child.clone())?;
        parent.set_tiling_direction(parent.tiling_direction().inverse());
      }
    } else {
      let split_children = tiling_children
        .into_iter()
        .filter_map(|child| child.as_split().cloned())
        .collect::<Vec<_>>();

      for split_child in split_children.iter().filter(|split_child| {
        split_child.tiling_direction() == parent.tiling_direction()
      }) {
        // Additionally flatten redundant top-level split containers in
        // the child.
        if split_child.child_count() == 1 {
          if let Some(split_grandchild) =
            split_child.children()[0].as_split()
          {
            flatten_split_container(split_grandchild.clone())?;
          }
        }

        flatten_split_container(split_child.clone())?;
      }
    }
  }

  Ok(())
}
