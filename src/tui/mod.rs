//! The interactive interface.

pub mod app;
pub mod event;
pub mod render;

use std::path::Path;
use std::time::Duration;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::error::{Error, Result};
use crate::fsx;
use crate::prompt;
use crate::tui::app::{App, Effect, Level};

/// The terminal used by the interface.
pub type Backend = CrosstermBackend<std::io::Stdout>;
type Term = Terminal<Backend>;

/// Runs the interactive interface until the user quits.
pub fn run(mut app: App) -> Result<()> {
    let mut terminal = ratatui::try_init()?;

    if let Err(error) = terminal.backend_mut().execute(EnableMouseCapture) {
        let _ = ratatui::try_restore();
        return Err(Error::Io(error));
    }

    let outcome = event_loop(&mut terminal, &mut app);

    let _ = terminal.backend_mut().execute(DisableMouseCapture);
    let _ = ratatui::try_restore();

    outcome
}

/// The event loop of the interface.
fn event_loop(terminal: &mut Term, app: &mut App) -> Result<()> {
    let mut dirty = true;

    loop {
        // The screen is only painted when something changed, the rest of the
        // time the program waits for the events.
        if dirty {
            if let Err(error) = terminal.draw(|frame| render::render(frame, app)) {
                return Err(Error::Io(error));
            }
            dirty = false;
        }

        if !crossterm::event::poll(Duration::from_millis(200))? {
            continue;
        }

        let event = crossterm::event::read()?;
        if let Some(effect) = event::handle(app, event) {
            match effect {
                Effect::OpenEditor(path) => open_with_editor(terminal, app, &path)?,
                Effect::ExtractAll(destination) => extract_all(terminal, app, &destination)?,
            }
        }

        dirty = true;

        if app.quit {
            break;
        }
    }

    Ok(())
}

/// Opens a file with the editor, leaving the interface while it runs.
fn open_with_editor(terminal: &mut Term, app: &mut App, path: &Path) -> Result<()> {
    let Some(editor) = app.editor.clone() else {
        app.set_status(
            "no editor found, define one in the configuration file or the EDITOR variable",
            Level::Warning,
        );
        return Ok(());
    };

    suspend(terminal)?;

    let outcome = editor.open(path);

    resume(terminal)?;

    match outcome {
        Ok(status) if !status.success() => {
            app.set_status(
                format!("the editor exited with the status {status}"),
                Level::Warning,
            );
        }
        Ok(_) => {}
        Err(error) => app.set_status(error.to_string(), Level::Error),
    }

    Ok(())
}

/// Extracts the whole archive into a directory.
fn extract_all(terminal: &mut Term, app: &mut App, destination: &Path) -> Result<()> {
    let staging = fsx::TempDir::new("content-7z-stage-")?;

    suspend(terminal)?;

    let mut outcome = extract_into(app, &staging, destination);

    // The password may be needed to extract the content of the archive.
    if matches!(outcome, Err(Error::PasswordRequired)) {
        if let Some(password) = prompt::read_password("Password: ")? {
            app.password = Some(password);
            outcome = extract_into(app, &staging, destination);
        }
    }

    resume(terminal)?;

    match outcome {
        Ok(()) => app.set_status(
            format!("the archive was extracted into {}", destination.display()),
            Level::Info,
        ),
        Err(Error::WrongPassword) => {
            app.password = None;
            app.set_status("the password is not correct", Level::Error);
        }
        Err(Error::PasswordRequired) => {
            app.set_status("the archive is encrypted", Level::Error);
        }
        Err(error) => app.set_status(error.to_string(), Level::Error),
    }

    Ok(())
}

/// Extracts the archive in a staging directory and copies it to its final
/// destination.
fn extract_into(app: &App, staging: &fsx::TempDir, destination: &Path) -> Result<()> {
    app.archive
        .extract_all(staging.path(), app.password.as_deref())?;
    fsx::remove_escaping_links(staging.path())?;
    std::fs::create_dir_all(destination)?;
    fsx::copy_tree_safely(staging.path(), destination)?;
    Ok(())
}

/// Leaves the alternate screen and restores the cooked mode.
fn suspend(terminal: &mut Term) -> Result<()> {
    terminal
        .backend_mut()
        .execute(crossterm::terminal::LeaveAlternateScreen)
        .map_err(Error::Io)?;
    disable_raw_mode().map_err(Error::Io)?;
    Ok(())
}

/// Enters the alternate screen again.
fn resume(terminal: &mut Term) -> Result<()> {
    enable_raw_mode().map_err(Error::Io)?;
    terminal
        .backend_mut()
        .execute(crossterm::terminal::EnterAlternateScreen)
        .map_err(Error::Io)?;

    // The editor may have left anything on the screen, the terminal is
    // resized to its own size to force a complete redraw. `Terminal::clear`
    // is not used because it asks the terminal for the cursor position, which
    // is not always possible.
    let size = terminal.size().map_err(Error::Io)?;
    terminal
        .resize(ratatui::layout::Rect::new(0, 0, size.width, size.height))
        .map_err(Error::Io)?;
    Ok(())
}
