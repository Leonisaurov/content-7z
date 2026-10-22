//! State of the interactive interface.

use std::path::{Path, PathBuf};

use ratatui::widgets::TableState;

use crate::archive::{Archive, ArchiveEntry};
use crate::config::Config;
use crate::editor::Editor;
use crate::error::{Error, Result};
use crate::fsx::TempDir;
use crate::path::join_within;
use crate::theme::Theme;
use crate::tree::{self, Node, NodeKind};

/// A row of the list, an entry of the current directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub encrypted: bool,
    /// Full path of the entry inside the archive.
    pub path: String,
}

/// Severity of a status message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Info,
    Warning,
    Error,
}

/// A message shown in the status bar.
#[derive(Debug, Clone)]
pub struct Status {
    pub text: String,
    pub level: Level,
}

/// The action a confirmation dialog confirms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    /// Continue with an extraction into a directory that is not empty.
    ExtractAll(PathBuf),
}

/// The action an input dialog confirms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAction {
    /// Ask for the password of the archive.
    Password,
    /// Ask for the destination of a full extraction.
    Destination,
}

/// A modal dialog.
#[derive(Debug, Clone)]
pub enum Dialog {
    Message {
        title: String,
        text: String,
        level: Level,
    },
    Confirm {
        title: String,
        text: String,
        action: ConfirmAction,
    },
    Input {
        title: String,
        prompt: String,
        value: String,
        masked: bool,
        action: InputAction,
    },
    Help,
}

/// An action that was waiting for something to happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pending {
    /// Open a file once the password is known.
    Open(String),
    /// Extract the whole archive once the password is known.
    ExtractAll(PathBuf),
}

/// Side effects the interface asks the program to perform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Open a file with the editor.
    OpenEditor(PathBuf),
    /// Extract the whole archive into a directory.
    ExtractAll(PathBuf),
}

/// The filter applied to the list of entries.
#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub query: String,
    pub editing: bool,
}

/// The state of the interactive interface.
pub struct App {
    pub archive: Archive,
    pub theme: Theme,
    pub config: Config,
    pub editor: Option<Editor>,

    tree: Node,
    path: Vec<String>,
    rows: Vec<Row>,
    pub state: TableState,

    pub filter: Filter,
    pub dialog: Option<Dialog>,
    pub status: Option<Status>,
    pub pending: Option<Pending>,
    pub password: Option<String>,
    pub temp: TempDir,
    pub quit: bool,
}

impl App {
    /// Builds the state for the given archive.
    pub fn new(archive: Archive, config: Config, password: Option<String>) -> Result<Self> {
        let theme = config.theme.clone();
        let editor = crate::editor::resolve(config.editor.as_deref());
        let mut tree = tree::build_tree(&archive.entries);
        tree::sort_children(&mut tree);

        let temp = TempDir::new("content-7z-")?;

        let mut app = Self {
            archive,
            theme,
            config,
            editor,
            tree,
            path: Vec::new(),
            rows: Vec::new(),
            state: TableState::new(),
            filter: Filter::default(),
            dialog: None,
            status: None,
            pending: None,
            password,
            temp,
            quit: false,
        };

        app.rebuild_rows();
        Ok(app)
    }

    /// Path of the directory that is currently displayed.
    pub fn path(&self) -> &[String] {
        &self.path
    }

    /// Breadcrumb of the directory that is currently displayed.
    pub fn breadcrumb(&self) -> String {
        if self.path.is_empty() {
            String::from("/")
        } else {
            format!("/{}", self.path.join("/"))
        }
    }

    /// The rows of the list.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// The node of the tree that is currently displayed.
    pub fn current_node(&self) -> &Node {
        let mut node = &self.tree;
        for component in &self.path {
            match node.child(component) {
                Some(child) => node = child,
                None => break,
            }
        }
        node
    }

    /// The selected row, if any.
    pub fn selected_row(&self) -> Option<&Row> {
        let index = self.state.selected()?;
        self.rows.get(index)
    }

    /// Recomputes the list of rows for the current directory.
    pub fn rebuild_rows(&mut self) {
        let query = self.filter.query.to_lowercase();
        let show_hidden = self.config.show_hidden;

        let mut rows: Vec<Row> = self
            .current_node()
            .children
            .iter()
            .filter(|child| show_hidden || !child.name.starts_with('.'))
            .filter(|child| query.is_empty() || child.name.to_lowercase().contains(&query))
            .map(|child| Row {
                name: child.name.clone(),
                is_dir: child.kind == NodeKind::Dir,
                size: child.size,
                encrypted: child.encrypted,
                path: self.path_of(child),
            })
            .collect();

        rows.sort_by(|left, right| match (left.is_dir, right.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
        });

        self.rows = rows;
        self.state
            .select(if self.rows.is_empty() { None } else { Some(0) });
    }

    /// Full path of a child of the current directory.
    fn path_of(&self, child: &Node) -> String {
        if self.path.is_empty() {
            child.name.clone()
        } else {
            format!("{}/{}", self.path.join("/"), child.name)
        }
    }

    /// Moves the selection by `delta` positions.
    pub fn move_selection(&mut self, delta: isize) {
        if self.rows.is_empty() {
            return;
        }

        let current = self.state.selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, self.rows.len() as isize - 1);
        self.state.select(Some(next as usize));
    }

    /// Selects the first or the last row.
    pub fn select_edge(&mut self, last: bool) {
        if self.rows.is_empty() {
            return;
        }
        let index = if last { self.rows.len() - 1 } else { 0 };
        self.state.select(Some(index));
    }

    /// Enters the selected directory.
    pub fn enter_selected(&mut self) {
        let Some(row) = self.selected_row() else {
            return;
        };
        if !row.is_dir {
            return;
        }

        let name = row.name.clone();
        self.path.push(name);
        self.filter = Filter::default();
        self.rebuild_rows();
    }

    /// Goes back to the parent directory.
    pub fn go_back(&mut self) {
        if self.path.pop().is_some() {
            self.filter = Filter::default();
            self.rebuild_rows();
        }
    }

    /// Starts the filtering.
    pub fn start_filter(&mut self) {
        self.filter.editing = true;
    }

    /// Adds a character to the filter.
    pub fn filter_push(&mut self, character: char) {
        self.filter.query.push(character);
        self.rebuild_rows();
    }

    /// Removes the last character of the filter.
    pub fn filter_backspace(&mut self) {
        self.filter.query.pop();
        self.rebuild_rows();
    }

    /// Stops the filtering, keeping the current query.
    pub fn filter_apply(&mut self) {
        self.filter.editing = false;
    }

    /// Removes the filter completely.
    pub fn filter_clear(&mut self) {
        self.filter.query.clear();
        self.filter.editing = false;
        self.rebuild_rows();
    }

    /// Shows a message in the status bar.
    pub fn set_status(&mut self, text: impl Into<String>, level: Level) {
        self.status = Some(Status {
            text: text.into(),
            level,
        });
    }

    /// Shows the help dialog.
    pub fn show_help(&mut self) {
        self.dialog = Some(Dialog::Help);
    }

    /// Closes the current dialog.
    pub fn close_dialog(&mut self) {
        self.dialog = None;
        self.pending = None;
    }

    /// Asks for the password of the archive.
    pub fn ask_password(&mut self) {
        self.dialog = Some(Dialog::Input {
            title: String::from("Password"),
            prompt: String::from("The archive is encrypted, enter its password"),
            value: String::new(),
            masked: true,
            action: InputAction::Password,
        });
    }

    /// Asks for the destination of a full extraction.
    pub fn ask_destination(&mut self) {
        self.dialog = Some(Dialog::Input {
            title: String::from("Extract"),
            prompt: String::from("Directory where the archive will be extracted"),
            value: String::from("./"),
            masked: false,
            action: InputAction::Destination,
        });
    }

    /// Handles the confirmation of a dialog.
    pub fn confirm(&mut self) -> Option<Effect> {
        let Some(Dialog::Confirm { action, .. }) = self.dialog.take() else {
            return None;
        };

        match action {
            ConfirmAction::ExtractAll(destination) => Some(Effect::ExtractAll(destination)),
        }
    }

    /// Handles the submission of an input dialog.
    pub fn submit_input(&mut self) -> Option<Effect> {
        let Some(Dialog::Input { value, action, .. }) = self.dialog.take() else {
            return None;
        };

        match action {
            InputAction::Password => {
                self.password = Some(value);
                self.run_pending()
            }
            InputAction::Destination => {
                let destination = PathBuf::from(value.trim());
                let destination = if destination.as_os_str().is_empty() {
                    PathBuf::from(".")
                } else {
                    destination
                };

                if self.archive.encrypted && self.password.is_none() {
                    // The password is needed to extract the content.
                    self.pending = Some(Pending::ExtractAll(destination));
                    self.ask_password();
                    return None;
                }

                if destination.exists()
                    && std::fs::read_dir(&destination)
                        .map(|mut entries| entries.next().is_some())
                        .unwrap_or(false)
                {
                    self.dialog = Some(Dialog::Confirm {
                        title: String::from("Extract"),
                        text: format!(
                            "{} is not empty, the existing files will be replaced.",
                            destination.display()
                        ),
                        action: ConfirmAction::ExtractAll(destination),
                    });
                    return None;
                }

                Some(Effect::ExtractAll(destination))
            }
        }
    }

    /// Runs the action that was waiting for a password.
    fn run_pending(&mut self) -> Option<Effect> {
        let pending = self.pending.take()?;
        match pending {
            Pending::Open(path) => self.open_path(&path),
            Pending::ExtractAll(destination) => Some(Effect::ExtractAll(destination)),
        }
    }

    /// Opens the selected file with the editor.
    pub fn request_open(&mut self) -> Option<Effect> {
        let row = self.selected_row()?.clone();
        if row.is_dir {
            return None;
        }

        if self.editor.is_none() {
            self.set_status(
                "no editor found, define one in the configuration file or the EDITOR variable",
                Level::Warning,
            );
            return None;
        }

        if self.archive.encrypted && self.password.is_none() {
            self.pending = Some(Pending::Open(row.path.clone()));
            self.ask_password();
            return None;
        }

        self.open_path(&row.path)
    }

    /// Extracts a file (when needed) and returns the effect to open it.
    fn open_path(&mut self, path: &str) -> Option<Effect> {
        let relative = PathBuf::from(path);
        let Some(target) = join_within(self.temp.path(), &relative) else {
            self.set_status("the entry has an unsafe path", Level::Error);
            return None;
        };

        let cached = target.exists();
        if cached && !self.config.always_overwrite {
            return Some(Effect::OpenEditor(target));
        }

        let Some(entry) = self.entry_of(path) else {
            self.set_status(format!("{path} is not part of the archive"), Level::Error);
            return None;
        };

        match self
            .archive
            .extract_entry(&entry, self.temp.path(), self.password.as_deref())
        {
            Ok(extracted) => Some(Effect::OpenEditor(extracted)),
            Err(Error::WrongPassword) => {
                self.password = None;
                self.pending = Some(Pending::Open(path.to_string()));
                self.set_status("the password is not correct", Level::Error);
                self.ask_password();
                None
            }
            Err(Error::PasswordRequired) => {
                self.pending = Some(Pending::Open(path.to_string()));
                self.ask_password();
                None
            }
            Err(error) => {
                self.set_status(error.to_string(), Level::Error);
                None
            }
        }
    }

    /// Finds the archive entry that matches a path.
    fn entry_of(&self, path: &str) -> Option<ArchiveEntry> {
        self.archive
            .entries
            .iter()
            .find(|entry| entry.components.join("/") == path)
            .cloned()
    }

    /// Reloads the archive from disk.
    pub fn reload(&mut self) -> Result<()> {
        let archive = Archive::open(&self.archive.path, self.password.as_deref())?;
        let mut tree = tree::build_tree(&archive.entries);
        tree::sort_children(&mut tree);

        self.archive = archive;
        self.tree = tree;
        self.path.clear();
        self.filter.query.clear();
        self.rebuild_rows();
        Ok(())
    }

    /// Path of the temporary directory used for the extracted files.
    pub fn temp_path(&self) -> &Path {
        self.temp.path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::ArchiveKind;

    fn archive_with(entries: Vec<ArchiveEntry>) -> Archive {
        Archive {
            path: PathBuf::from("test.7z"),
            kind: ArchiveKind::SevenZip,
            entries,
            encrypted: false,
        }
    }

    fn app_with(entries: Vec<ArchiveEntry>) -> App {
        App::new(archive_with(entries), Config::default(), None).unwrap()
    }

    #[test]
    fn lists_the_entries_of_the_root() {
        let app = app_with(vec![
            ArchiveEntry::new("folder", true, 0, false).unwrap(),
            ArchiveEntry::new("file.txt", false, 10, false).unwrap(),
        ]);

        assert_eq!(app.rows().len(), 2);
        // Directories come first.
        assert!(app.rows()[0].is_dir);
        assert_eq!(app.breadcrumb(), "/");
    }

    #[test]
    fn navigates_into_folders() {
        let app = app_with(vec![
            ArchiveEntry::new("folder", true, 0, false).unwrap(),
            ArchiveEntry::new("folder/inner.txt", false, 10, false).unwrap(),
        ]);

        let mut app = app;
        app.state.select(Some(0));
        app.enter_selected();

        assert_eq!(app.path(), ["folder".to_string()]);
        assert_eq!(app.rows().len(), 1);
        assert_eq!(app.rows()[0].name, "inner.txt");
        assert_eq!(app.breadcrumb(), "/folder");
    }

    #[test]
    fn goes_back_to_the_parent() {
        let mut app = app_with(vec![
            ArchiveEntry::new("folder", true, 0, false).unwrap(),
            ArchiveEntry::new("folder/inner.txt", false, 10, false).unwrap(),
        ]);

        app.state.select(Some(0));
        app.enter_selected();
        app.go_back();

        assert!(app.path().is_empty());
        assert_eq!(app.rows().len(), 1);
    }

    #[test]
    fn filters_the_entries() {
        let mut app = app_with(vec![
            ArchiveEntry::new("alpha.txt", false, 1, false).unwrap(),
            ArchiveEntry::new("beta.txt", false, 1, false).unwrap(),
        ]);

        app.filter_push('b');
        assert_eq!(app.rows().len(), 1);
        assert_eq!(app.rows()[0].name, "beta.txt");

        app.filter_backspace();
        assert_eq!(app.rows().len(), 2);
    }

    #[test]
    fn selection_stays_inside_the_list() {
        let mut app = app_with(vec![ArchiveEntry::new("a", false, 1, false).unwrap()]);
        app.move_selection(-5);
        assert_eq!(app.state.selected(), Some(0));
        app.move_selection(5);
        assert_eq!(app.state.selected(), Some(0));
    }

    #[test]
    fn an_encrypted_archive_asks_for_the_password() {
        let mut app = app_with(vec![ArchiveEntry::new("file.txt", false, 1, true).unwrap()]);
        app.archive.encrypted = true;
        app.editor = Some(Editor {
            program: String::from("editor"),
            args: Vec::new(),
        });
        app.state.select(Some(0));

        assert!(app.request_open().is_none());
        assert!(matches!(app.dialog, Some(Dialog::Input { .. })));
        assert_eq!(app.pending, Some(Pending::Open("file.txt".to_string())));
    }

    #[test]
    fn the_password_is_used_for_the_pending_action() {
        let mut app = app_with(vec![ArchiveEntry::new("file.txt", false, 1, true).unwrap()]);
        app.archive.encrypted = true;
        app.editor = Some(Editor {
            program: String::from("editor"),
            args: Vec::new(),
        });
        app.state.select(Some(0));
        app.request_open();

        // The password is typed and confirmed.
        if let Some(Dialog::Input { value, .. }) = app.dialog.as_mut() {
            value.push_str("secret");
        }
        let effect = app.submit_input();

        assert_eq!(app.password.as_deref(), Some("secret"));
        // The file does not exist in the archive listing, the extraction
        // fails, but the password was stored.
        assert!(app.password.is_some());
        let _ = effect;
    }

    #[test]
    fn a_file_is_not_extracted_without_an_editor() {
        let mut app = app_with(vec![ArchiveEntry::new("file.txt", false, 1, false).unwrap()]);
        app.editor = None;
        app.state.select(Some(0));

        assert!(app.request_open().is_none());
        assert!(app.status.is_some());
    }

    #[test]
    fn the_extraction_asks_for_the_password_of_an_encrypted_archive() {
        let mut app = app_with(vec![ArchiveEntry::new("file.txt", false, 1, true).unwrap()]);
        app.archive.encrypted = true;
        app.ask_destination();

        if let Some(Dialog::Input { value, .. }) = app.dialog.as_mut() {
            value.clear();
            value.push_str("/tmp/output");
        }
        assert!(app.submit_input().is_none());
        assert_eq!(
            app.pending,
            Some(Pending::ExtractAll(PathBuf::from("/tmp/output")))
        );
        assert!(matches!(app.dialog, Some(Dialog::Input { .. })));
    }
}
