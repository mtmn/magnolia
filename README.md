# fzf-nav

is a utility for tracking and querying file system navigation history.

## Installation

```bash
# Build
cargo build --release

# Initialize database
./db/init

# Source shell integration
source ./misc/fzf-nav
```

**Dependencies**: `fzf`, `fd`, `sqlite3`

## Usage

```bash
fzf-nav [--db-path <path>] [--no-color] <command> [limit]
```

### Commands

- `recent-dirs [50]` - Recent directory visits
- `recent-files [50]` - Recent file opens  
- `popular-dirs [50]` - Most visited directories
- `file-stats` - File type usage statistics
- `search <query>` - Search history

### Shell Functions

- `d` - Fuzzy navigate to directory
- `f` - Fuzzy open file
- `rd` - Recent directories
- `rf` - Recent files

## File Handling

The `f()` function opens files based on extension:
- **Media** (mp4, mp3, etc.) → `mpv`
- **Images** (jpg, png, etc.) → `nsxiv`
- **Documents** (pdf, epub) → `sioyek`  
- **Other files** → `$EDITOR`

## Examples

```bash
# Navigate to recent directory
d

# Open recent file
f

# Show popular directories
fzf-nav popular-dirs 10

# Search for rust files
fzf-nav search rust
```

## Database

SQLite database with two tables:
- `directory_history` - path, timestamp
- `file_history` - path, file_type, action, timestamp

Default location: `~/.fzf.db` (configurable with `--db-path`)
