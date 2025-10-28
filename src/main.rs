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

    command: String,
}

fn run_command(command: &str) {
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
                println!("success");
            } else {
                println!("failed: {}", output.status);
            }
        }
        Err(e) => println!("error: {}", e),
    }
    println!("---\n");
}

fn main() {
    let args = Args::parse();

    let glob = Glob::new(&args.pattern)
        .expect("invalid glob pattern")
        .compile_matcher();

    println!("watching {} (pattern: {})", args.dir, args.pattern);
    println!("command: {}\n", args.command);

    run_command(&args.command);

    let (tx, rx) = std::sync::mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_millis(500), tx)
        .expect("failed to create debouncer");

    debouncer
        .watcher()
        .watch(Path::new(&args.dir), RecursiveMode::Recursive)
        .expect("failed to watch");

    for result in rx {
        match result {
            Ok(events) => {
                let matched = events.iter().any(|e| glob.is_match(&e.path));
                
                if matched {
                    run_command(&args.command);
                }
            }
            Err(e) => println!("error: {:?}", e),
        }
    }
}