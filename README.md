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
`
-p, --pattern <PATTERN> | patterns to watch
`

`
-d, --dir <DIR> | directory to watch [default: .]
`

`
--debounce <MS> | debounce delay in ms [default: 500]
`

`
-c, --clear | clear screen before running
`

`
--config <FILE> | config file [default: .watchrun.toml]
`

## examples

watch rust files:

```bash
watch-run -p "**/*.rs" "cargo test"
```

watch web stuff:

```bash
watch-run -p "**/*.html" -p "**/*.css" -p "**/*.js" "npm run build"
```

custom debounce:

```bash
watch-run -p "src/**/*" --debounce 1000 "make build"
```