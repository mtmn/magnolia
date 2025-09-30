# fzf-nav

Navigation auditing tool.

## Usage
webp soon.

## Motivation
Since my workflow tends to share similarities with that of a squirrel trying to hide a hazelnut and I have [declared bankruptcy on my mind](https://www.youtube.com/watch?v=XUZ9VATeF_4) some time ago, this made sense.

## Installation

```bash
# Build
cargo build --release

# Initialize database
./db/init

# Source shell integration
source ./shell/fzf-nav
```

**Dependencies**: `fzf`, `fd`, `sqlite3`

## Usage

```bash
fzf-nav [--db-path <path>] [--no-color] <command> [limit]
```

### Commands

- `recent-dirs [500]` - Recent directory visits
- `recent-files [500]` - Recent file opens  
- `change-to-dir [1000]` - Interactive recent file opens  
- `change-to-file [1000]` - Interactive recent file opens  
- `popular-dirs [500]` - Most visited directories
- `file-stats` - File type usage statistics
- `search <query>` - Search history

### Shell Functions

- `d` - Fuzzy navigate to directory
- `f` - Fuzzy open file
- `rd` - Interactive recent directories
- `rf` - Interactive recent files
- `dg` - Recent directories in fzf
- `fg` - Recent files in fzf, open in $EDITOR

## File Handling

The `f()` function opens files based on extension:
- **Media** (mp4, mp3, etc.) → `mpv`
- **Images** (jpg, png, etc.) → `nsxiv`
- **Documents** (pdf, epub) → `sioyek`  
- **Other files** → `$EDITOR`

## Examples

```bash
# Navigate to directory
d

# Open recent file
f

# Interactively navigate to recent directory
dg

# Interactively open recent file (if you frequently use `fg`, consider a different function name)
fg

# Show popular directories
fzf-nav popular-dirs 10

# Search for rust files
fzf-nav search rust

# Interactively change to dir
fzf-nav change-to-dir

# Interactively change to file
fzf-nav change-to-file
```

## Database

SQLite database with two tables:
- `directory_history` - path, timestamp
- `file_history` - path, file_type, action, timestamp

Default location: `~/.fzf.db` (configurable with `--db-path`)
