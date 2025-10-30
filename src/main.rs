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

    #[arg(short, long, num_args = 0..)]
    ignore: Vec<String>,

    command: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct Config {
    #[serde(default = "default_dir")]
    dir: String,
    #[serde(default)]
    patterns: Vec<String>,
    #[serde(default)]
    commands: Vec<String>,
    #[serde(default)]
    command: Option<String>,
    #[serde(default = "default_debounce")]
    debounce: u64,
    #[serde(default)]
    clear: bool,
    #[serde(default)]
    ignore: Vec<String>,
}

impl Config {
    fn get_commands(&self) -> Vec<String> {
        if !self.commands.is_empty() {
            self.commands.clone()
        } else if let Some(cmd) = &self.command {
            vec![cmd.clone()]
        } else {
            vec![]
        }
    }

    fn build_globsets(&self, extra_patterns: &[String], extra_ignores: &[String]) -> Result<(GlobSet, GlobSet), globset::Error> {
        let mut watch_builder = GlobSetBuilder::new();
        let mut ignore_builder = GlobSetBuilder::new();

        if !extra_patterns.is_empty() {
            for pattern in extra_patterns {
                watch_builder.add(Glob::new(pattern)?);
            }
        } else if !self.patterns.is_empty() {
            for pattern in &self.patterns {
                watch_builder.add(Glob::new(pattern)?);
            }
        } else {
            watch_builder.add(Glob::new("**/*")?);
        }

        for pattern in &self.ignore {
            ignore_builder.add(Glob::new(pattern)?);
        }

        for pattern in extra_ignores {
            ignore_builder.add(Glob::new(pattern)?);
        }

        let default_ignores = vec![
            "**/target/**",
            "**/node_modules/**",
            "**/.git/**",
            "**/.watchrun.toml",
        ];

        for pattern in default_ignores {
            ignore_builder.add(Glob::new(pattern)?);
        }

        Ok((watch_builder.build()?, ignore_builder.build()?))
    }
}

fn default_dir() -> String {
    ".".to_string()
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

fn run_commands(commands: &[String], should_clear: bool) {
    if should_clear {
        clear_screen();
    }

    for command in commands {
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
                    break;
                }
            }
            Err(e) => {
                println!("{} {}", "error:".red(), e);
                break;
            }
        }
    }
    
    println!("{}", "---".dimmed());
    println!("{}", "waiting...".dimmed());
}

fn main() {
    let args = Args::parse();

    let config = load_config(&args.config).unwrap_or_else(|| Config {
        dir: default_dir(),
        patterns: vec![],
        commands: vec![],
        command: None,
        debounce: default_debounce(),
        clear: false,
        ignore: vec![],
    });

    let dir = args.dir
        .unwrap_or_else(|| config.dir.clone());
    
    let commands = if !args.command.is_empty() {
        args.command
    } else {
        config.get_commands()
    };

    if commands.is_empty() {
        eprintln!("{} no command specifed", "error:".red());
        exit(1);
    }

    let debounce = args.debounce
        .unwrap_or(config.debounce);

    let clear = args.clear || config.clear;

    let watch_path = Path::new(&dir);
    if !watch_path.exists() {
        eprintln!("{} directory doesnt exist", "error:".red());
        exit(1);
    }

    let (watch_globset, ignore_globset) = match config.build_globsets(&args.pattern, &args.ignore) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{} invalid pattern: {}", "error:".red(), e);
            exit(1);
        }
    };

    let display_patterns = if !args.pattern.is_empty() {
        args.pattern.clone()
    } else if !config.patterns.is_empty() {
        config.patterns.clone()
    } else {
        vec!["**/*".to_string()]
    };

    println!("{} {}", "watching:".blue(), dir);
    println!("{} {}", "patterns:".blue(), display_patterns.join(", ").yellow());
    println!("{} {}", "commands:".blue(), commands.join(" && "));
    println!("{} {}ms", "debounce:".blue(), debounce);

    run_commands(&commands, clear);

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
                    .filter(|e| {
                        watch_globset.is_match(&e.path) && !ignore_globset.is_match(&e.path)
                    })
                    .collect();

                if !matched_files.is_empty() {
                    println!("\n{} {}", "changed:".yellow(), matched_files[0].path.display());
                    run_commands(&commands, clear);
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
    fn test_build_globsets() {
        let config = Config {
            dir: ".".to_string(),
            patterns: vec!["**/*.rs".to_string()],
            commands: vec![],
            command: None,
            debounce: 500,
            clear: false,
            ignore: vec!["**/test/**".to_string()],
        };
        
        let result = config.build_globsets(&[], &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_commands_priority() {
        let config = Config {
            dir: ".".to_string(),
            patterns: vec![],
            commands: vec!["cmd1".to_string(), "cmd2".to_string()],
            command: Some("old_cmd".to_string()),
            debounce: 500,
            clear: false,
            ignore: vec![],
        };
        
        assert_eq!(config.get_commands(), vec!["cmd1", "cmd2"]);
    }

    #[test]
    fn test_get_commands_fallback() {
        let config = Config {
            dir: ".".to_string(),
            patterns: vec![],
            commands: vec![],
            command: Some("fallback_cmd".to_string()),
            debounce: 500,
            clear: false,
            ignore: vec![],
        };
        
        assert_eq!(config.get_commands(), vec!["fallback_cmd"]);
    }

    #[test]
    fn defaults_work() {
        assert_eq!(default_dir(), ".");
        assert_eq!(default_debounce(), 500);
    }
}