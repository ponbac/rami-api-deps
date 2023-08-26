use std::path::PathBuf;

use clap::Parser;
use console::style;
use rami_api_deps::pipeline::Pipeline;
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

    // cwd + root_dir
    let root_dir = std::env::current_dir().unwrap().join(&args.root_dir);

    let pipeline_walker = WalkDir::new(&root_dir)
        .into_iter()
        // Filter out any non-accessible files
        .filter_map(|e| e.ok())
        // Only include pipeline files
        .filter(is_pipeline_file);

    let mut pipelines = Vec::new();
    for entry in pipeline_walker {
        let pipeline = Pipeline::new(entry.path().to_path_buf());

        println!(
            "Pipeline {}, includes {} projects.",
            style(&pipeline.name).green().italic().bold(),
            style(&pipeline.projects.len()).yellow().bold()
        );
        println!(
            "Path filter: {}",
            style(&pipeline.complete_path_filter())
                .cyan()
                .italic()
                .bold()
        );
        println!();

        pipelines.push(pipeline);
    }

    // create path filter files next to the pipeline files
    println!(
        "{}",
        style("Creating path filter files...").magenta().bold()
    );
    for pipeline in pipelines {
        let output_dir = pipeline.path.parent().unwrap();
        let file = output_dir.join(".azure-pathfilter");

        std::fs::write(file, pipeline.complete_path_filter()).unwrap();
    }
    println!(
        "{} {}",
        style("Done!").green().bold(),
        style("Now it's time to paste the path filters into Azure DevOps.").italic()
    );
}

fn is_pipeline_file(entry: &DirEntry) -> bool {
    entry.file_type().is_file()
        && entry.path().file_name().unwrap_or_default() == "azure-pipelines.yml"
}
