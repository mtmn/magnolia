# magnolia
Interactive shell navigation and history based on [fzf](https://github.com/junegunn/fzf) and [sqlite](https://www.sqlite.org/index.html).

## Motivation
My workflow resembles a squirrel trying to hide a hazelnut so I have [declared bankruptcy on my mind](https://www.youtube.com/watch?v=XUZ9VATeF_4) and making this made sense.

## Installation

```bash
# Install dependencies
sudo pacman -Sy fd fzf rustup sqlite3 --needed

# If you use Homebrew
brew install fd fzf rustup sqlite3

# Install Rust toolchain
rustup install stable

# Set default toolchain to stable
rustup default stable

# Build
cargo build --release

# Initialize database
./db/init

# Source shell integration
source ./shell/magnolia
```

## Usage

```bash
magnolia [--db-path <path>] [--no-color] <command> [limit]
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
- `cd` - Navigate to directory
- `rd` - Interactive recent directories
- `rf` - Interactive recent files
- `dg` - Recent directories in fzf
- `fg` - Recent files in fzf, open in vim

## File Handling

The `f()` function opens files based on extension:

- audio `mpv`
- video `mpv`
- other `vim`

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
magnolia popular-dirs 10

# Search for rust files
magnolia search rust

# Interactively change to dir
magnolia change-to-dir

# Interactively change to file
magnolia change-to-file
```

## Database

SQLite database with two tables:
- `directory_history` - path, timestamp
- `file_history` - path, file_type, action, timestamp

Default location: `~/.magnolia.db` (configurable with `--db-path`)
