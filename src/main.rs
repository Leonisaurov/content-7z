//! content-7z: browse the content of 7z, zip and tar archives.

use std::process::ExitCode;

use content_7z::archive::Archive;
use content_7z::cli::{self, Command, Options};
use content_7z::error::{Error, Result};
use content_7z::fsx;
use content_7z::prompt;
use content_7z::tree::{self, Node, NodeKind};
use content_7z::tui::app::App;
use content_7z::util;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}: {error}", cli::PROGRAM);
            ExitCode::FAILURE
        }
    }
}

/// Runs the program.
fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let command = cli::parse(&args).map_err(Error::Config)?;

    match command {
        Command::Help => {
            println!("{}", cli::help());
            Ok(())
        }
        Command::Version => {
            println!("{} {}", cli::PROGRAM, cli::VERSION);
            Ok(())
        }
        Command::List(options) => list(options),
        Command::Extract {
            options,
            destination,
        } => extract(options, &destination),
        Command::Browse(options) => browse(options),
    }
}

/// Prints the content of the archive.
fn list(options: Options) -> Result<()> {
    let archive = Archive::open(&options.archive, options.password.as_deref())?;

    let mut tree = tree::build_tree(&archive.entries);
    tree::sort_children(&mut tree);

    println!("{}", util::file_name_of(&archive.path));
    print_children(&tree, 1);

    Ok(())
}

/// Prints the children of a node, indented.
fn print_children(node: &Node, depth: usize) {
    let indent = "  ".repeat(depth);
    for child in &node.children {
        let size = if child.kind == NodeKind::Dir {
            String::new()
        } else {
            format!("  {}", util::human_size(child.size))
        };
        println!("{indent}{}{size}", child.name);
        print_children(child, depth + 1);
    }
}

/// Extracts the whole archive into a directory.
fn extract(options: Options, destination: &std::path::Path) -> Result<()> {
    let archive = Archive::open(&options.archive, options.password.as_deref())?;

    let staging = fsx::TempDir::new("content-7z-stage-")?;
    archive.extract_all(staging.path(), options.password.as_deref())?;
    fsx::remove_escaping_links(staging.path())?;

    std::fs::create_dir_all(destination)?;
    fsx::copy_tree_safely(staging.path(), destination)?;

    println!(
        "{} was extracted into {}",
        util::file_name_of(&archive.path),
        destination.display()
    );

    Ok(())
}

/// Opens the archive in the interactive interface.
fn browse(options: Options) -> Result<()> {
    let mut password = options.password.clone();

    // The listing of an encrypted archive may need the password, ask for it
    // until the archive can be opened.
    let archive = loop {
        match Archive::open(&options.archive, password.as_deref()) {
            Ok(archive) => break archive,
            Err(Error::PasswordRequired) | Err(Error::WrongPassword) => {
                let entered = prompt::read_password(&format!(
                    "The archive {} is encrypted, password: ",
                    util::file_name_of(&options.archive)
                ))?
                .ok_or(Error::Cancelled)?;
                password = Some(entered);
            }
            Err(error) => return Err(error),
        }
    };

    // The content of the archive may be encrypted even when its listing is
    // not, ask for the password before it is needed.
    if archive.encrypted && password.is_none() {
        if let Some(entered) = prompt::read_password(&format!(
            "The content of {} is encrypted, password: ",
            util::file_name_of(&options.archive)
        ))? {
            if !entered.is_empty() {
                password = Some(entered);
            }
        }
    }

    let config = content_7z::config::Config::load()?;
    let app = App::new(archive, config, password)?;

    content_7z::tui::run(app)
}
