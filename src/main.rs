use clap::Parser;
use colored::*;
use globset::Glob;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::path::Path;
use std::process::{Command, exit};
use std::time::Duration;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "**/*")]
    pattern: String,

    #[arg(short, long, default_value = ".")]
    dir: String,

    #[arg(long, default_value = "500")]
    debounce: u64,

    #[arg(short, long)]
    clear: bool,

    command: String,
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

fn main() {
    let args = Args::parse();

    let watch_path = Path::new(&args.dir);
    if !watch_path.exists() {
        eprintln!("{} directory '{}' doesnt exist", "error:".red(), args.dir);
        exit(1);
    }
    if !watch_path.is_dir() {
        eprintln!("{} '{}' is not a directory", "error:".red(), args.dir);
        exit(1);
    }

    let glob = match Glob::new(&args.pattern) {
        Ok(g) => g.compile_matcher(),
        Err(e) => {
            eprintln!("{} invalid pattern: {}", "error:".red(), e);
            exit(1);
        }
    };

    println!("{} {} ({})", "watching:".blue(), args.dir, args.pattern.yellow());
    println!("{} {}", "command:".blue(), args.command);
    println!("{} {}ms", "debounce:".blue(), args.debounce);

    run_command(&args.command, args.clear);

    let (tx, rx) = std::sync::mpsc::channel();

    let mut debouncer = match new_debouncer(Duration::from_millis(args.debounce), tx) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{} {}", "failed to create watcher:".red(), e);
            exit(1);
        }
    };

    if let Err(e) = debouncer.watcher().watch(watch_path, RecursiveMode::Recursive) {
        eprintln!("{} {}", "failed to watch:".red(), e);
        exit(1);
    }

    println!("{}\n", "started, ctrl+c to stop".green());

    for result in rx {
        match result {
            Ok(events) => {
                let matched_files: Vec<_> = events
                    .iter()
                    .filter(|e| glob.is_match(&e.path))
                    .collect();

                if !matched_files.is_empty() {
                    println!("\n{} {}", "changed:".yellow(), matched_files[0].path.display());
                    run_command(&args.command, args.clear);
                }
            }
            Err(e) => eprintln!("{} {:?}", "watch error:".yellow(), e),
        }
    }
}