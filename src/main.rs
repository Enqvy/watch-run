use clap::Parser;
use globset::Glob;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::path::Path;
use std::process::Command;
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

    println!("\nrunning: {}", command);
    println!("---");

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
                println!("done");
            } else {
                println!("failed: {}", output.status);
            }
        }
        Err(e) => println!("error: {}", e),
    }
    println!("---");
    println!("waiting for changes...\n");
}

fn main() {
    let args = Args::parse();

    let glob = Glob::new(&args.pattern)
        .expect("invalid pattern")
        .compile_matcher();

    println!("watching {} ({})", args.dir, args.pattern);
    println!("debounce: {}ms", args.debounce);
    println!("command: {}\n", args.command);

    run_command(&args.command, args.clear);

    let (tx, rx) = std::sync::mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_millis(args.debounce), tx)
        .expect("failed to create debouncer");

    debouncer
        .watcher()
        .watch(Path::new(&args.dir), RecursiveMode::Recursive)
        .expect("failed to watch");

    for result in rx {
        match result {
            Ok(events) => {
                let matched_files: Vec<_> = events
                    .iter()
                    .filter(|e| glob.is_match(&e.path))
                    .collect();

                if !matched_files.is_empty() {
                    println!("\nchanged: {}", matched_files[0].path.display());
                    run_command(&args.command, args.clear);
                }
            }
            Err(e) => println!("error: {:?}", e),
        }
    }
}