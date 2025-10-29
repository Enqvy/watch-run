use clap::Parser;
use colored::*;
use globset::{Glob, GlobSet, GlobSetBuilder};
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::process::{Command, exit};
use std::time::Duration;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, num_args = 1..)]
    pattern: Vec<String>,

    #[arg(short, long)]
    dir: Option<String>,

    #[arg(long)]
    debounce: Option<u64>,

    #[arg(short, long)]
    clear: bool,

    #[arg(long, default_value = ".watchrun.toml")]
    config: String,

    command: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Config {
    #[serde(default = "default_dir")]
    dir: String,
    #[serde(default = "default_patterns")]
    patterns: Vec<String>,
    #[serde(default)]
    command: Option<String>,
    #[serde(default = "default_debounce")]
    debounce: u64,
    #[serde(default)]
    clear: bool,
}

fn default_dir() -> String {
    ".".to_string()
}

fn default_patterns() -> Vec<String> {
    vec!["**/*".to_string()]
}

fn default_debounce() -> u64 {
    500
}

fn load_config(path: &str) -> Option<Config> {
    if Path::new(path).exists() {
        let content = fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    } else {
        None
    }
}

fn clear_screen() {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "cls"]).status().ok();
    } else {
        Command::new("clear").status().ok();
    }
}

fn run_command(command: &str, should_clear: bool) {
    if should_clear {
        clear_screen();
    }

    println!("\n{} {}", "running:".cyan(), command);
    println!("{}", "---".dimmed());

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", command]).output()
    } else {
        Command::new("sh").arg("-c").arg(command).output()
    };

    match output {
        Ok(output) => {
            print!("{}", String::from_utf8_lossy(&output.stdout));
            print!("{}", String::from_utf8_lossy(&output.stderr));

            if output.status.success() {
                println!("{}", "done".green());
            } else {
                println!("{} {}", "failed:".red(), output.status);
            }
        }
        Err(e) => println!("{} {}", "error:".red(), e),
    }
    println!("{}", "---".dimmed());
    println!("{}", "waiting...".dimmed());
}

fn build_globset(patterns: &[String]) -> Result<GlobSet, globset::Error> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    builder.build()
}

fn main() {
    let args = Args::parse();

    let config = load_config(&args.config);

    let dir = args.dir
        .or_else(|| config.as_ref().map(|c| c.dir.clone()))
        .unwrap_or_else(default_dir);
    
    let patterns = if !args.pattern.is_empty() {
        args.pattern
    } else {
        config.as_ref()
            .map(|c| c.patterns.clone())
            .unwrap_or_else(default_patterns)
    };

    let command = args.command
        .or_else(|| config.as_ref().and_then(|c| c.command.clone()));
    
    let command = match command {
        Some(cmd) => cmd,
        None => {
            eprintln!("{} no command specified", "error:".red());
            exit(1);
        }
    };

    let debounce = args.debounce
        .or_else(|| config.as_ref().map(|c| c.debounce))
        .unwrap_or_else(default_debounce);

    let clear = args.clear || config.as_ref().map(|c| c.clear).unwrap_or(false);

    let watch_path = Path::new(&dir);
    if !watch_path.exists() {
        eprintln!("{} directory doesnt exist", "error:".red());
        exit(1);
    }

    let globset = match build_globset(&patterns) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{} invalid pattern: {}", "error:".red(), e);
            exit(1);
        }
    };

    println!("{} {}", "watching:".blue(), dir);
    println!("{} {}", "patterns:".blue(), patterns.join(", ").yellow());
    println!("{} {}", "command:".blue(), command);
    println!("{} {}ms", "debounce:".blue(), debounce);

    run_command(&command, clear);

    let (tx, rx) = std::sync::mpsc::channel();

    let mut debouncer = match new_debouncer(Duration::from_millis(debounce), tx) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{} {}", "failed:".red(), e);
            exit(1);
        }
    };

    if let Err(e) = debouncer.watcher().watch(watch_path, RecursiveMode::Recursive) {
        eprintln!("{} {}", "failed:".red(), e);
        exit(1);
    }

    println!("{}\n", "started".green());

    for result in rx {
        match result {
            Ok(events) => {
                let matched_files: Vec<_> = events
                    .iter()
                    .filter(|e| globset.is_match(&e.path))
                    .collect();

                if !matched_files.is_empty() {
                    println!("\n{} {}", "changed:".yellow(), matched_files[0].path.display());
                    run_command(&command, clear);
                }
            }
            Err(e) => eprintln!("{} {:?}", "error:".yellow(), e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_globset() {
        let patterns = vec!["**/*.rs".to_string(), "**/*.toml".to_string()];
        let result = build_globset(&patterns);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_glob() {
        let patterns = vec!["[invalid".to_string()];
        let result = build_globset(&patterns);
        assert!(result.is_err());
    }

    #[test]
    fn defaults_work() {
        assert_eq!(default_dir(), ".");
        assert_eq!(default_patterns(), vec!["**/*"]);
        assert_eq!(default_debounce(), 500);
    }
}