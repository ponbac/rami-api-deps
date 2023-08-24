use std::path::PathBuf;

use clap::Parser;
use rami_api_deps::{pipeline::Pipeline, project::Project};
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

    // let csproj_walker = WalkDir::new(&args.root_dir)
    //     .into_iter()
    //     // Filter out any non-accessible files
    //     .filter_map(|e| e.ok())
    //     // Only include .csproj files
    //     .filter(is_csproj_file);

    // for entry in csproj_walker {
    //     let project = Project::new(entry.path().to_path_buf());

    //     project.pretty_print();
    //     println!();
    // }

    let pipeline_walker = WalkDir::new(&args.root_dir)
        .into_iter()
        // Filter out any non-accessible files
        .filter_map(|e| e.ok())
        // Only include pipeline files
        .filter(is_pipeline_file);

    for entry in pipeline_walker {
        let pipeline = Pipeline::new(entry.path().to_path_buf());

        pipeline.pretty_print();
        println!();
    }
}

fn is_csproj_file(entry: &DirEntry) -> bool {
    entry.file_type().is_file() && entry.path().extension().unwrap_or_default() == "csproj"
}

fn is_pipeline_file(entry: &DirEntry) -> bool {
    entry.file_type().is_file()
        && entry.path().file_name().unwrap_or_default() == "azure-pipelines.yml"
}
