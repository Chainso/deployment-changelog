use std::fmt::Display;

use clap::Parser;

#[derive(Parser)]
struct Cli {
    start_commit: String,
    end_commit: String
}

fn main() {
    println!("Hello, world!");
    let args = Cli::parse();
}
