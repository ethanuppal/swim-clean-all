use indicatif::{HumanBytes, ProgressBar};
use owo_colors::OwoColorize;
use snafu::{prelude::*, Whatever};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    time::Duration,
};
use walkdir::WalkDir;

use argh::FromArgs;

/// Recursively clean all swim projects in a given directory that match the specified criteria
#[derive(FromArgs)]
struct Args {
    /// directories to skip when traversing
    #[argh(option)]
    skip: Vec<PathBuf>,

    /// maximum depth search limit; defaults to 100
    #[argh(option, default = "100")]
    max_depth: usize,

    /// the root directory to recursively search for swim projects; defaults to the cwd
    #[argh(positional, default = "PathBuf::from(\".\")")]
    search_root: PathBuf,
}

fn driver() -> Result<(), Whatever> {
    let args = argh::from_env::<Args>();

    let search_root = fs::canonicalize(&args.search_root).whatever_context(format!(
        "Failed to canonicalize search root {}",
        args.search_root.to_string_lossy()
    ))?;

    let mut skipped_directories = Vec::with_capacity(args.skip.len());
    for skipped_directory in args.skip {
        skipped_directories.push(fs::canonicalize(&skipped_directory).whatever_context(
            format!(
                "Failed to canonicalize skipped directory {}",
                skipped_directory.to_string_lossy()
            ),
        )?);
    }

    let spinner = ProgressBar::new_spinner().with_message("Scanning for swim projects");
    spinner.enable_steady_tick(Duration::from_millis(100));

    let projects = WalkDir::new(search_root)
        .max_depth(args.max_depth)
        .into_iter()
        .filter_entry(|entry| {
            !skipped_directories
                .iter()
                .any(|skipped_directory| entry.path().starts_with(skipped_directory))
        })
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().is_dir()
                && entry.path().join("swim.toml").exists()
                && entry.path().join("build").exists()
        })
        .collect::<Vec<_>>();

    spinner.finish_and_clear();

    if projects.is_empty() {
        println!("No swim projects found");
        return Ok(());
    }

    println!(
        "{}",
        format!(
            "{} swim project{} found:",
            projects.len(),
            if projects.len() == 1 { "" } else { "s" }
        )
        .bold()
        .green()
    );
    println!();

    let mut size_saved = 0;
    for project in projects {
        let potential_savings =
            fs_extra::dir::get_size(project.path()).whatever_context(format!(
                "Failed to get size of directory {}",
                project.path().to_string_lossy()
            ))?;

        print!(
            "{}",
            format!(
                "  Clean {}? ({}) [y/n] ",
                project.path().to_string_lossy(),
                HumanBytes(potential_savings)
            )
            .bold()
            .blue()
        );

        io::stdout()
            .flush()
            .whatever_context("Failed to flush stdout to show cleanup CLI")?;

        let user_answer = io::stdin()
            .lines()
            .next()
            .unwrap()
            .whatever_context("Failed to read line from stdin")?;

        crossterm::execute!(io::stdout(), crossterm::cursor::MoveToPreviousLine(1))
            .whatever_context("Failed to move up one line")?;

        if matches!(user_answer.trim(), "y" | "Y" | "yes") {
            fs::remove_dir_all(project.path().join("build")).whatever_context(format!(
                "Failed to remove build directory for project at {}",
                project.path().to_string_lossy()
            ))?;
            println!(
                "Cleaned {} ({}).",
                project.path().to_string_lossy(),
                HumanBytes(potential_savings)
            );
            size_saved += potential_savings;
        } else {
            println!(
                "{}",
                format!(
                    "Skipped {} ({}).",
                    project.path().to_string_lossy(),
                    HumanBytes(potential_savings)
                )
                .dimmed()
            )
        }
    }

    println!();
    if size_saved > 0 {
        println!("{} successfully cleaned", HumanBytes(size_saved));
    } else {
        println!("No projects cleaned");
    }

    Ok(())
}

fn main() {
    if let Err(error) = driver() {
        eprintln!("{}", error);
    }
}
