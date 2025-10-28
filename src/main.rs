use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "watch-run")]
#[command(about = "Run commands when files change", long_about = None)]
struct Args {
    // example: "src/**/*.rs"
    pattern: String,
    
    command: String,
}

fn main() {
    let args = Args::parse();
    println!("Watching: {}", args.pattern);
    println!("Will run: {}", args.command);
}