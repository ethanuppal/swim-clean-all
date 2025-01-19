//! This program is free software: you can redistribute it and/or modify it
//! under the terms of the GNU General Public License as published by the
//! Free Software Foundation, either version 3 of the License, or (at your
//! option) any later version.
//!
//! This program is distributed in the hope that it will be useful, but
//! WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
//! General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License along
//! with this program. If not, see <https://www.gnu.org/licenses/>.

use argh::FromArgs;
use indicatif::{HumanBytes, ProgressBar};
use owo_colors::OwoColorize;
use serde::Deserialize;
use snafu::{OptionExt, ResultExt, Whatever};
use std::{
    cmp, env, fs,
    io::{self, Write},
    iter,
    path::{Path, PathBuf},
    process,
    time::Duration,
};
use walkdir::WalkDir;

// swim forces this (for now)
const BUILD_DIRECTORY_PATH: &str = "build";

const DEFAULT_CONFIG_FILE_NAME: &str = "swim-clean-all.toml";

#[derive(Deserialize)]
struct Config {
    /// Directories to skip when traversing.
    skip: Option<Vec<PathBuf>>,
}

/// Tries to read a config file from `XDG_CONFIG_HOME`, then from the operating
/// system local default, finally trying ``~/.config`.
fn read_config(
    manually_specify: Option<&Path>,
) -> Result<Option<Config>, Whatever> {
    let Some(config_file_path) =
        manually_specify.map(|path| path.to_path_buf()).or_else(|| {
            let config_directory =
                env::var("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or(
                    dirs::config_local_dir()
                        .unwrap_or(PathBuf::from("~/.config")),
                );

            if config_directory.is_dir() {
                Some(config_directory.join(DEFAULT_CONFIG_FILE_NAME))
            } else {
                log::warn!("No config directory found on system");
                None
            }
        })
    else {
        return Ok(None);
    };

    if config_file_path.is_file() {
        let config_file_contents = fs::read_to_string(config_file_path)
            .whatever_context("Failed to load config file")?;
        Ok(Some(
            toml::from_str(&config_file_contents)
                .whatever_context("Failed to parse config file")?,
        ))
    } else {
        log::warn!(
            "Config file {} does not exist or is not a file",
            config_file_path.to_string_lossy()
        );
        Ok(None)
    }
}

/// Recursively clean all swim projects in a given directory that match the
/// specified criteria
#[derive(FromArgs)]
struct Opts {
    /// directories to skip when traversing
    #[argh(option)]
    skip: Vec<PathBuf>,

    /// maximum depth search limit; defaults to 100
    #[argh(option, default = "100")]
    max_depth: usize,

    /// manually specify a config path, e.g., foo.toml
    #[argh(option)]
    config: Option<PathBuf>,

    /// do not load and extend the config file
    #[argh(switch)]
    ignore_config: bool,

    /// print debugging information
    #[argh(switch)]
    verbose: bool,

    /// the root directory to recursively search for swim projects; defaults to
    /// the cwd
    #[argh(positional, default = "PathBuf::from(\".\")")]
    search_root: PathBuf,
}

fn parse_opts() -> Result<Opts, Whatever> {
    let mut args = env::args();
    let command_name = args
        .next()
        .whatever_context("Missing command name in argument list")?;
    // A bug in swim (https://gitlab.com/spade-lang/swim/-/blob/2a386a16b0fb3e2ba3a075e073279b25f97d6b56/src/main.rs#L414)
    // means that the first real argument will actually be the command name. I'm
    // not going to be too smart about this since the bug will probably be
    // fixed soon (although it means you can't specify a directory called
    // "clean-all" as the first argument. I've submitted a patch in !153.
    let mut passed_args = vec![];
    let first_arg = args.next().whatever_context(
        "Missing first argument; try --help for usage information",
    )?;
    if first_arg.as_str() == "clean-all" {
    } else {
        passed_args.push(first_arg.as_str());
    }

    let args = args.collect::<Vec<_>>();
    passed_args.extend(args.iter().map(String::as_str));

    match Opts::from_args(&[&command_name], &passed_args) {
        Ok(mut opts) => {
            if opts.verbose {
                colog::init();
            }

            if !opts.ignore_config {
                if let Some(config) = read_config(opts.config.as_deref())
                    .whatever_context(
                        "Failed to load config file if one exists",
                    )?
                {
                    if let Some(skip) = config.skip {
                        opts.skip.extend(skip);
                    }
                }
            }

            log::info!("Searching in: {}", opts.search_root.to_string_lossy());
            log::info!(
                "Skipping directories: {}",
                opts.skip
                    .iter()
                    .map(|skipped| skipped.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            Ok(opts)
        }
        Err(early_exit) => {
            print!("{}", early_exit.output);
            process::exit(0)
        }
    }
}

fn canonicalize(path: &Path) -> io::Result<PathBuf> {
    let mut path = path.to_path_buf();
    if path.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            let mut path_with_home = home;
            path_with_home.extend(path.components().skip(1));
            path = path_with_home;
        }
    }
    fs::canonicalize(path)
}

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let opts = parse_opts()
        .whatever_context("Failed to parse command line arguments")?;

    let search_root =
        canonicalize(&opts.search_root).whatever_context(format!(
            "Failed to canonicalize search root {}",
            opts.search_root.to_string_lossy()
        ))?;

    let mut skipped_directories = Vec::with_capacity(opts.skip.len());
    for skipped_directory in opts.skip {
        skipped_directories.push(
            canonicalize(&skipped_directory).whatever_context(format!(
                "Failed to canonicalize skipped directory {}",
                skipped_directory.to_string_lossy()
            ))?,
        );
    }

    let spinner =
        ProgressBar::new_spinner().with_message("Scanning for swim projects");
    spinner.enable_steady_tick(Duration::from_millis(100));

    let minimum_components_to_show =
        search_root.components().collect::<Vec<_>>().len();

    let projects = WalkDir::new(search_root)
        .max_depth(opts.max_depth)
        .into_iter()
        .filter_entry(|entry| {
            !skipped_directories.iter().any(|skipped_directory| {
                entry.path().starts_with(skipped_directory)
            })
        })
        .filter_map(|entry| entry.ok())
        .inspect(|entry| {
            let components = entry.path().components().collect::<Vec<_>>();
            let display_components = cmp::min(
                minimum_components_to_show + 2,
                if entry.path().is_dir() {
                    components.len()
                } else {
                    components.len() - 1
                },
            );
            let display_directory = components
                .into_iter()
                .take(display_components)
                .map(|component| {
                    let component =
                        component.as_os_str().to_string_lossy().to_string();
                    if component.starts_with("/") {
                        component
                    } else {
                        format!("{}/", component)
                    }
                })
                .collect::<Vec<_>>()
                .join("");
            spinner.set_message(format!(
                "Scanning for cleanable swim projects {}{}{}",
                "[".bold(),
                display_directory.bold(),
                "]".bold()
            ));
        })
        .filter(|entry| {
            entry.path().is_dir()
                && entry.path().join("swim.toml").exists()
                && entry.path().join(BUILD_DIRECTORY_PATH).exists()
        })
        .collect::<Vec<_>>();

    spinner.finish_and_clear();

    if projects.is_empty() {
        println!(
            "No cleanable swim projects found in {}",
            opts.search_root.to_string_lossy()
        );
        return Ok(());
    }

    let mut project_build_sizes = vec![];
    for project in &projects {
        project_build_sizes.push(
            fs_extra::dir::get_size(project.path().join(BUILD_DIRECTORY_PATH))
                .whatever_context(format!(
                    "Failed to get size of directory {}",
                    project.path().to_string_lossy()
                ))?,
        );
    }

    println!(
        "{}",
        format!(
            "{} cleanable swim project{} found (totalling {} potential savings)",
            projects.len(),
            if projects.len() == 1 { "" } else { "s" },
            HumanBytes(project_build_sizes.iter().sum())
        )
        .bold()
        .green()
    );
    println!();

    let mut size_saved = 0;
    for (project, potential_savings) in iter::zip(projects, project_build_sizes)
    {
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

        crossterm::execute!(
            io::stdout(),
            crossterm::cursor::MoveToPreviousLine(1)
        )
        .whatever_context("Failed to move up one line")?;

        if matches!(user_answer.trim(), "y" | "Y" | "yes") {
            fs::remove_dir_all(project.path().join(BUILD_DIRECTORY_PATH))
                .whatever_context(format!(
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
