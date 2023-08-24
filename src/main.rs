use std::path::PathBuf;

use clap::Parser;
use console::style;
use walkdir::{DirEntry, WalkDir};

/// Generate dependency things!
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Root directory to search from
    #[arg(short, long, default_value = ".")]
    root_dir: PathBuf,
}

// clear; cargo run -- --root-dir C:\Users\pbac\Dev\ramirent\SE-CustomerPortal
fn main() {
    let args = Args::parse();

    let walker = WalkDir::new(args.root_dir)
        .into_iter()
        // Filter out any non-accessible files
        .filter_map(|e| e.ok())
        // Only include .csproj files
        .filter(is_csproj_file);

    for (i, entry) in walker.enumerate() {
        let path = entry.path();
        if path.is_file() {
            println!(
                "{}{}: {}",
                style("file ").bold(),
                style(i).bold().cyan(),
                style(path.display()).dim().italic()
            );
        }
    }
}

fn is_csproj_file(entry: &DirEntry) -> bool {
    entry.file_type().is_file() && entry.path().extension().unwrap_or_default() == "csproj"
}
