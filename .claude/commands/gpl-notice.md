Check all .rs files that have been modified in this session or are staged in git for GPL-3.0 copyright headers. For any file missing the header, prepend the following block:

```
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

Steps:
1. Run `git diff --name-only HEAD -- '*.rs'` to find modified .rs files
2. For each file, check if the first line starts with `// Copyright`
3. If not, prepend the header block above (followed by a blank line before the existing code)
4. Report which files were updated
