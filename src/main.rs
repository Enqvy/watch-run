use clap::Parser;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::time::Duration;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(name = "watch-run")]
struct Args {
    #[arg(default_value = ".")]
    path: String,
    
    command: String,
}

fn main() {
    let args = Args::parse();
    
    println!("watching {}", args.path);
    println!("command: {}", args.command);
    
    let (tx, rx) = std::sync::mpsc::channel();
    
    let mut debouncer = new_debouncer(Duration::from_millis(500), tx)
        .expect("failed to create debouncer");
    
    debouncer
        .watcher()
        .watch(Path::new(&args.path), RecursiveMode::Recursive)
        .expect("failed to watch path");
    
    println!("started, waiting for changes...\n");
    
    for result in rx {
        match result {
            Ok(events) => {
                for event in events {
                    println!("changed: {:?}", event.path);
                }
            }
            Err(e) => println!("error: {:?}", e),
        }
    }
}