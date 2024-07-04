use crate::{
    files::folder::Folder,
    window::{
        cursor::Cursor,
        scheme::Scheme,
        handler::Handler,
    },
    zip_manager::manager
};
use std::io::{StdoutLock, Write};
use crossterm::{terminal, QueueableCommand};
use config::Config;

pub struct Window<'a> {
    pub root: Folder,
    pub current: Vec<Folder>,
    pub width: u16,
    pub height: u16,
    pub scroll_x: u16,
    pub scroll_y: u16,
    pub scroll_change: bool,
    pub path_change: bool,
    pub on_dialog: bool,
    pub cursor: Cursor,
    pub path: String,
    pub scheme: Scheme,
    pub job: Option<Handler>,
    pub writer: *mut StdoutLock<'a>,
}

impl<'a> Drop for Window<'a> {
    fn drop(&mut self) {
        terminal::disable_raw_mode().unwrap();
        unsafe {
            (&mut (*self.writer)).queue(terminal::LeaveAlternateScreen).unwrap();
        }
    }
}

impl<'a> Window<'a> {
    pub fn new(stdout: *mut StdoutLock<'a>, config: Config) -> Self {
        let out = unsafe { &mut (*stdout) };
        out.queue(terminal::EnterAlternateScreen).unwrap();
        out.queue(terminal::EndSynchronizedUpdate).unwrap();
        terminal::enable_raw_mode().expect("Error al abrir la patalla");
        out.flush().unwrap();

        let (width, height) = terminal::size().unwrap();

        Self {
            root: Folder::new(""),
            current: vec![],
            width, 
            height,
            scroll_x: 0,
            scroll_y: 0,
            scroll_change: false,
            path_change: false,
            on_dialog: false,
            cursor: Cursor { x: 1, y: 4, need_update: false },
            path: String::new(),
            scheme: Scheme::from(config),
            job: None,
            writer: stdout,
        }
    }

    pub fn assign_root(&mut self, folder: Folder) {
        self.root = folder.clone();
        self.current = vec![folder];
    }

    pub fn assing_manager(&mut self, manager: manager::ZipManager) {
        self.assign_path(manager.get_path());
        self.assign_root(manager.get_root());
    } 

    pub fn get_current(&self) -> &Folder {
        if self.current.len() == 0 {

        }
        &self.current[self.current.len() - 1]
    }
    
    pub fn set_current(&mut self, folder: Folder) {
        self.current.push(folder);
        self.cursor.need_update = true;

        self.path_change = true;

        self.scroll_change = true;
        self.scroll_y = 0;
        self.scroll_x = 0;
    }

    pub fn back_current(&mut self) {
        if self.current.len() > 1 {
            self.current.pop().unwrap();

            self.path_change = true;
            self.scroll_change = true;
        }
    }

    pub fn plain_current(&self) -> String {
        let mut plain = String::from("");
        let mut s_flag = true;
        for current in &self.current {
            if s_flag {
                s_flag = false;
                continue;
            }
            plain += "/";
            plain += current.name.as_str();
        }
        plain
    }

    pub fn assign_path(&mut self, path: String) {
        self.path = path;
    }

    pub fn get_writer(&self) -> &mut StdoutLock<'a> {
        unsafe {
            &mut (*self.writer)
        }
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    } 

    pub fn move_up(&mut self) {
        if self.cursor.y > 4 {
            self.cursor.y -= 1;
            self.cursor.need_update = true;
        } else if self.scroll_y > 0 {
            self.scroll_y -= 1;
            self.scroll_change = true;
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor.y < self.height - 2 {
            self.cursor.y += 1;
            self.cursor.need_update = true;
        } else {
            self.scroll_y += 1;
            self.scroll_change = true;
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor.x > 1 {
            self.cursor.x -= 1;
            self.cursor.need_update = true;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor.x < self.width - 2 {
            self.cursor.x += 1;
            self.cursor.need_update = true;
        }
    }

    pub fn set_cursor(&mut self, x: u16, y: u16) {
        self.cursor.x = x;
        self.cursor.y = y;
        self.cursor.need_update = true;
    }

    pub fn set_scheme(&mut self, scheme: Scheme) {
        self.scheme = scheme
    }
}

