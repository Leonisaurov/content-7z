//! Reading a password from the terminal without echoing it.

use std::io::{self, BufRead, Write};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::tty::IsTty;

use crate::error::{Error, Result};

/// Asks the user for a password.
///
/// When the standard input is a terminal the characters are read in raw mode
/// and never echoed, otherwise a plain line is read from the input.
pub fn read_password(prompt: &str) -> Result<Option<String>> {
    if io::stdin().is_tty() {
        read_from_terminal(prompt)
    } else {
        read_from_pipe()
    }
}

/// Reads the password from an interactive terminal.
fn read_from_terminal(prompt: &str) -> Result<Option<String>> {
    let mut stdout = io::stdout();
    write!(stdout, "{prompt}")?;
    stdout.flush()?;

    enable_raw_mode()?;

    let mut password = String::new();
    let result = loop {
        match event::read() {
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Enter => break Ok(()),
                KeyCode::Esc => break Err(Error::Cancelled),
                KeyCode::Backspace => {
                    password.pop();
                    write!(stdout, "\u{8} \u{8}")?;
                    stdout.flush()?;
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    break Err(Error::Cancelled)
                }
                KeyCode::Char(character) => {
                    password.push(character);
                    write!(stdout, "*")?;
                    stdout.flush()?;
                }
                _ => {}
            },
            Ok(_) => {}
            Err(error) => break Err(Error::Io(error)),
        }
    };

    // The terminal has to be restored even when the read failed.
    let _ = disable_raw_mode();
    writeln!(stdout)?;
    stdout.flush()?;

    result?;

    Ok(Some(password))
}

/// Reads the password from a pipe.
fn read_from_pipe() -> Result<Option<String>> {
    let mut line = String::new();
    let read = io::stdin().lock().read_line(&mut line)?;
    if read == 0 {
        return Ok(None);
    }
    Ok(Some(line.trim_end_matches(['\r', '\n']).to_string()))
}
