use clap::Parser;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

#[derive(Parser, Debug)]
struct Args {
    #[arg(default_value = ".")]
    path: String,
    command: String,
}

fn run_command(command: &str) {
    println!("\nrunning: {}", command);
    println!("---");

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", command])
            .output()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
    };

    match output {
        Ok(output) => {
            print!("{}", String::from_utf8_lossy(&output.stdout));
            print!("{}", String::from_utf8_lossy(&output.stderr));
            
            if output.status.success() {
                println!("success");
            } else {
                println!("failed with: {}", output.status);
            }
        }
        Err(e) => println!("error executing: {}", e),
    }
    println!("---\n");
}

fn main() {
    let args = Args::parse();

    println!("watching: {}", args.path);
    println!("command: {}\n", args.command);
    
    run_command(&args.command);

    let (tx, rx) = std::sync::mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_millis(500), tx)
        .expect("failed to create debouncer");

    debouncer
        .watcher()
        .watch(Path::new(&args.path), RecursiveMode::Recursive)
        .expect("failed to watch path");

    for result in rx {
        match result {
            Ok(_) => {
                run_command(&args.command);
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}