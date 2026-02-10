#!/usr/bin/env bash
# Adds GPL-3.0 copyright headers to Rust source files.
# Usage:
#   ./scripts/add-gpl-header.sh [file ...]
#   If no files given, processes all staged .rs files (git diff --cached).

set -euo pipefail

HEADER='// Copyright (C) 2024 glzr-io <https://github.com/glzr-io>
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
'

add_header() {
  local file="$1"
  # Skip if header already present
  if head -1 "$file" | grep -q "^// Copyright"; then
    return 0
  fi
  echo "Adding GPL header: $file"
  local tmp
  tmp=$(mktemp)
  printf '%s\n' "$HEADER" | cat - "$file" > "$tmp"
  mv "$tmp" "$file"
}

if [ $# -gt 0 ]; then
  files=("$@")
else
  # Default: all staged .rs files
  mapfile -t files < <(git diff --cached --name-only --diff-filter=ACM -- '*.rs')
fi

for f in "${files[@]}"; do
  [ -f "$f" ] && add_header "$f"
done
