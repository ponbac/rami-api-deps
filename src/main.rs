use std::path::PathBuf;

use clap::Parser;
use rami_api_deps::project::Project;
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

    for entry in walker {
        let project = Project::new(entry.path().to_path_buf());

        project.pretty_print();
        println!();
    }
}

fn is_csproj_file(entry: &DirEntry) -> bool {
    entry.file_type().is_file() && entry.path().extension().unwrap_or_default() == "csproj"
}
