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
use wm_common::{VecDequeExt, WmEvent};

use super::{
  attach_container, detach_container, flatten_child_split_containers,
  set_focused_descendant,
};
use crate::{models::Container, traits::CommonGetters, wm_state::WmState};

/// Move a container to a new location in the tree. This detaches the
/// container from its current parent and attaches it to the new parent at
/// the specified index.
///
/// If this container is a tiling container, its siblings are resized on
/// detach, and the container is sized to the default tiling size with its
/// new siblings. No changes to the container's tiling size are made if
/// its parent stays the same.
///
/// This will flatten any redundant split containers after moving the
/// container, which can cause the target parent to become detached. For
/// example, in the layout V[1 H[2]] where container 1 is moved down, the
/// parent gets removed resulting in V[1 2].
pub fn move_container_within_tree(
  container_to_move: &Container,
  target_parent: &Container,
  target_index: usize,
  state: &WmState,
) -> anyhow::Result<()> {
  // Create iterator of parent, grandparent, and great-grandparent.
  let ancestors =
    container_to_move.ancestors().take(3).collect::<Vec<_>>();

  // Get lowest common ancestor (LCA) between `container_to_move` and
  // `target_parent`. This could be the `target_parent` itself.
  let lowest_common_ancestor =
    lowest_common_ancestor(container_to_move, target_parent)
      .context("No common ancestor between containers.")?;

  // If the container is already a child of the target parent, then shift
  // it to the target index.
  if container_to_move.parent().context("No parent.")? == *target_parent {
    target_parent
      .borrow_children_mut()
      .shift_to_index(target_index, container_to_move.clone());

    if container_to_move.has_focus(None) {
      state.emit_event(WmEvent::FocusedContainerMoved {
        focused_container: container_to_move.to_dto()?,
      });
    }

    return Ok(());
  }

  // Handle case where target parent is the LCA. For example, when swapping
  // sibling containers or moving a container to a direct ancestor.
  if *target_parent == lowest_common_ancestor {
    return move_to_lowest_common_ancestor(
      container_to_move,
      &lowest_common_ancestor,
      target_index,
      state,
    );
  }

  // Get ancestor of `container_to_move` that is a direct child of the LCA.
  // This could be the `container_to_move` itself.
  let container_to_move_ancestor = container_to_move
    .self_and_ancestors()
    .find(|ancestor| {
      ancestor.parent() == Some(lowest_common_ancestor.clone())
    })
    .context("Failed to get ancestor of container to move.")?;

  // Likewise, get ancestor of `target_parent` that is a direct child of
  // the LCA.
  let target_parent_ancestor = target_parent
    .self_and_ancestors()
    .find(|ancestor| {
      ancestor.parent() == Some(lowest_common_ancestor.clone())
    })
    .context("Failed to get ancestor of target parent.")?;

  // Get whether the container is the focused descendant in its original
  // subtree from the LCA.
  let is_focused_descendant = *container_to_move
    == container_to_move_ancestor
    || container_to_move
      .has_focus(Some(container_to_move_ancestor.clone()));

  // Get whether the ancestor of `container_to_move` appears before
  // `target_parent`'s ancestor in the child focus order of the LCA.
  let original_focus_index = container_to_move_ancestor.focus_index();
  let is_subtree_focused =
    original_focus_index < target_parent_ancestor.focus_index();

  detach_container(container_to_move.clone())?;
  attach_container(
    &container_to_move.clone(),
    &target_parent.clone(),
    Some(target_index),
  )?;

  // Set `container_to_move` as focused descendant within target subtree if
  // its original subtree had focus more recently (even if the container is
  // not the last focused within that subtree).
  if is_subtree_focused {
    set_focused_descendant(
      container_to_move,
      Some(&target_parent_ancestor),
    );
  }

  // If the focused descendant is moved to the targets subtree, then the
  // target's ancestor should be placed before the original ancestor in
  // LCA's child focus order.
  if is_focused_descendant && is_subtree_focused {
    lowest_common_ancestor
      .borrow_child_focus_order_mut()
      .shift_to_index(original_focus_index, target_parent_ancestor.id());
  }

  // After moving the container, flatten any redundant split containers.
  // For example, in the layout V[1 H[2]] where container 1 is moved down
  // to become V[H[1 2]], this will then need to be flattened to V[1 2].
  for ancestor in ancestors.iter().rev() {
    flatten_child_split_containers(ancestor)?;
  }

  if container_to_move.has_focus(None) {
    state.emit_event(WmEvent::FocusedContainerMoved {
      focused_container: container_to_move.to_dto()?,
    });
  }

  Ok(())
}

fn move_to_lowest_common_ancestor(
  container_to_move: &Container,
  lowest_common_ancestor: &Container,
  target_index: usize,
  state: &WmState,
) -> anyhow::Result<()> {
  // Keep reference to focus index of container's ancestor in LCA's child
  // focus order.
  let original_focus_index = container_to_move
    .self_and_ancestors()
    .find(|ancestor| {
      ancestor.parent() == Some(lowest_common_ancestor.clone())
    })
    .map(|ancestor| ancestor.focus_index())
    .context("Failed to get focus index of container's ancestor.")?;

  detach_container(container_to_move.clone())?;

  attach_container(
    &container_to_move.clone(),
    &lowest_common_ancestor.clone(),
    Some(target_index),
  )?;

  lowest_common_ancestor
    .borrow_child_focus_order_mut()
    .shift_to_index(original_focus_index, container_to_move.id());

  if container_to_move.has_focus(None) {
    state.emit_event(WmEvent::FocusedContainerMoved {
      focused_container: container_to_move.to_dto()?,
    });
  }

  Ok(())
}

/// Gets the lowest container in the tree that has both `container_a` and
/// `container_b` as descendants.
pub fn lowest_common_ancestor(
  container_a: &Container,
  container_b: &Container,
) -> Option<Container> {
  let mut ancestor_a = Some(container_a.clone());

  // Traverse upwards from container A.
  while let Some(current_ancestor_a) = ancestor_a {
    let mut ancestor_b = Some(container_b.clone());

    // Traverse upwards from container B.
    while let Some(current_ancestor_b) = ancestor_b {
      if current_ancestor_a == current_ancestor_b {
        return Some(current_ancestor_a);
      }

      ancestor_b = current_ancestor_b.parent();
    }

    ancestor_a = current_ancestor_a.parent();
  }

  None
}

#[cfg(test)]
mod tests {
  use tokio::sync::mpsc;
  use wm_common::TilingDirection;
  use wm_platform::Dispatcher;

  use super::move_container_within_tree;
  use crate::{
    models::{Container, SplitContainer, TilingWindow, Workspace},
    traits::CommonGetters,
    wm_state::WmState,
  };

  fn mock_state() -> WmState {
    let (event_tx, _) = mpsc::unbounded_channel();
    let (exit_tx, _) = mpsc::unbounded_channel();
    WmState::new(Dispatcher::mock(), event_tx, exit_tx)
  }

  fn split(
    tiling_direction: TilingDirection,
    child: impl Into<crate::models::TilingContainer>,
  ) -> SplitContainer {
    SplitContainer::mock()
      .tiling_direction(tiling_direction)
      .tiling_containers(vec![child.into()])
      .call()
  }

  fn assert_no_splits(workspace: &Workspace, expected_children: usize) {
    let children = workspace.children();
    assert_eq!(children.len(), expected_children);
    assert!(children.iter().all(|child| !child.is_split()));
  }

  #[test]
  fn moving_only_child_out_of_nested_split_leaves_no_empty_split() {
    let state = mock_state();
    let window = TilingWindow::mock().call();
    let nested = split(TilingDirection::Horizontal, window.clone());

    let workspace = Workspace::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![
        TilingWindow::mock().call().into(),
        split(TilingDirection::Vertical, nested).into(),
      ])
      .call();

    let workspace_container: Container = workspace.clone().into();
    move_container_within_tree(
      &window.into(),
      &workspace_container,
      workspace.child_count(),
      &state,
    )
    .unwrap();

    assert_no_splits(&workspace, 2);
  }

  #[test]
  fn moving_only_child_out_of_deep_split_chain_leaves_no_empty_split() {
    let state = mock_state();
    let window = TilingWindow::mock().call();
    let innermost = split(TilingDirection::Vertical, window.clone());
    let middle = split(TilingDirection::Horizontal, innermost);

    let workspace = Workspace::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![
        TilingWindow::mock().call().into(),
        split(TilingDirection::Vertical, middle).into(),
      ])
      .call();

    let workspace_container: Container = workspace.clone().into();
    move_container_within_tree(
      &window.into(),
      &workspace_container,
      workspace.child_count(),
      &state,
    )
    .unwrap();

    assert_no_splits(&workspace, 2);
  }
}
