# recall
Search and act on shell command history with optional context (timestamps, working directories) and regex filtering.

Rust CLI and a shell wrapper.

## Install
```bash
curl -fsSL https://raw.githubusercontent.com/myersm0/recall/main/install.sh | sh
```

Then add to your shell profile (`.bashrc`, `.zshrc`, etc.):

```bash
source "$HOME/.local/share/recall/recall.sh"
```

## Usage
```
recall [pattern] [flags]
```

Displays a numbered menu of matching commands from your history. Type a number to yank that command to clipboard.

### Examples

```bash
recall                  # browse recent history
recall git              # commands matching "git"
recall -p               # commands run under the current directory tree
recall -d myproject     # commands run in directories matching "myproject"
recall docker -n 25     # show up to 25 results
recall -v               # verbose mode -- include timestamps and directories
recall -H 5000          # recall with a history context size of 5000 (how deep in history to read)
```

### Flags

| Flag | Description |
|------|-------------|
| `-r <regex>` | Filter by command pattern (same as positional arg) |
| `-p` | Restrict to current directory tree |
| `-d <regex>` | Filter by directory pattern |
| `-n <number>` | Max results (default 15) |
| `-v` | Show timestamps and directories |

## How it works

The shell wrapper installs a `PROMPT_COMMAND` hook that logs each command to `~/.bash_history_extended` as CSV with base64-encoded fields.

The binary reads from the tail of this file, so performance stays constant as the file grows. By default the most recent 1000 items of history will be considered. Use option `-H $n` to expand/reduce this to read `$n` lines instead.

## Config

Optional. Place at `~/.config/recall/config.toml`:

```toml
history_path = "~/.bash_history_extended"
```
