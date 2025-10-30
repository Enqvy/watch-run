# watch-run

simple file watcher that runs commands when stuff changes

## install

```bash
cargo install --path .
```

## usage

basic:

```bash
watch-run "cargo test"
```

with patterns:

```bash
watch-run -p "**/*.rs" -p "Cargo.toml" "cargo build"
```

with config file - create `.watchrun.toml`:
```toml
dir = "."
patterns = ["**/*.rs", "Cargo.toml"]
command = "cargo test"
debounce = 500
clear = true
```

## options

`-p, --pattern <PATTERN>` - patterns to watch (can be used multiple times)

`-d, --dir <DIR>` - directory to watch [default: .]

`-i, --ignore <PATTERN>` - patterns to ignore (can be used multiple times)

`--debounce <MS>` - debounce delay in ms [default: 500]

`-c, --clear` - clear screen before running

`--config <FILE>` - config file [default: .watchrun.toml]

## features

**runs immediately** - command runs once on startup, then on every change

**multiple commands** - runs commands in sequence, stops if one fails

**smart ignores** - automatically ignores `target/`, `node_modules/`, `.git/`, and `.watchrun.toml`

**custom ignores** - add your own ignore patterns via `-i` flag or config

## examples

watch rust files:

```bash
watch-run -p "**/*.rs" "cargo test"
```

watch web stuff:

```bash
watch-run -p "**/*.html" -p "**/*.css" -p "**/*.js" "npm run build"
```

ignore test files:

```bash
watch-run -p "**/*.rs" -i "**/*_test.rs" -i "**/tests/**" "cargo build"
```

custom debounce:

```bash
watch-run -p "src/**/*" --debounce 1000 "make build"
```

multiple commands in sequence:

```toml
# .watchrun.toml
patterns = ["**/*.rs"]
commands = ["cargo fmt", "cargo clippy", "cargo test"]
clear = true
```

build then deploy:

```toml
# .watchrun.toml
patterns = ["src/**/*"]
commands = ["npm run build", "npm run deploy"]
debounce = 1000
```

with custom ignores:

```toml
# .watchrun.toml
patterns = ["**/*.rs"]
ignore = ["**/benches/**", "**/examples/**"]
command = "cargo test"
```

## config file

all options available in `.watchrun.toml`:

```toml
dir = "."                          # directory to watch
patterns = ["**/*.rs"]             # patterns to watch
command = "cargo test"             # single command (legacy)
commands = ["cmd1", "cmd2"]        # multiple commands (runs in order)
debounce = 500                     # debounce in milliseconds
clear = true                       # clear screen before running
ignore = ["**/tmp/**"]             # custom ignore patterns
```

**note:** `commands` takes priority over `command` if both are specified

## default ignores

these patterns are always ignored:
- `**/target/**`
- `**/node_modules/**`
- `**/.git/**`
- `**/.watchrun.toml`

add more with `-i` flag or `ignore` in config