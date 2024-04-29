use std::process::{Command, exit};
use std::env::args;

use crossterm::{
    self, terminal::{self, Clear, ClearType}, cursor::MoveTo,
    QueueableCommand,
    event::{self, Event, KeyCode, KeyEvent, MouseEventKind, MouseButton}
};
use std::{time::Duration, io::{self, stdout, Stdout, Write, StdoutLock}, thread};

#[derive(Clone, Debug)]
enum Entry {
    File(String),
    Folder(Folder),
}

#[derive(Clone, Debug)]
enum EntryType {
    File,
    Folder
}

#[derive(Clone, Debug)]
struct Folder {
    name: String,
    content: Vec<Entry>,
}

impl Folder {
    fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            content: Vec::new()
        }
    }

    fn add_file(&mut self, file_name: &str) {
        self.content.push(Entry::File(file_name.to_string()));
    }

    fn add_folder(&mut self, folder: Folder) {
        self.content.push(Entry::Folder(folder));
    }

    fn get_folder(&mut self, folder_name: &str) -> Option<&mut Folder> {
        for entry in &mut self.content {
            if let Entry::Folder(folder) = entry {
                if folder.name == folder_name {
                    return Some(folder)
                }
            }
        }
        None
    }

    fn contain_entry(&mut self, entry_name: &str) -> bool {
        for entry in &self.content {
            match entry {
                Entry::File(file_name) => {
                    if file_name == entry_name {
                        return true
                    }
                },
                Entry::Folder(folder) => {
                    if folder.name == entry_name {
                        return true
                    }
                },
            }
        }
        false
    }

    fn add_entry(&mut self, entry: &str, file_type: &EntryType) {
        eprintln!("{}", entry);
        for i in 0..entry.len() {
            if let Some(character) = entry.get(i..i+1) {
                if character == "/" {
                    if let Some(path) = entry.get(0..i) {
                        if self.contain_entry(path) {
                            if let Some(sub_path) = entry.get(i+1..entry.len()) {
                                if let Some(folder) = self.get_folder(path) {
                                    folder.add_entry(sub_path, file_type);
                                } else {
                                    let mut new_entry = Folder::new(path);
                                    eprintln!("No folder found: {} / {}", path, sub_path);
                                    new_entry.add_entry(sub_path, file_type);
                                    self.add_folder(new_entry);
                                }
                            } else {
                                eprintln!("The subdir cannot be get");
                            }
                            eprintln!("In the Main Folder {}.", path);
                            return;
                        }
                        let mut new_entry = Folder::new(path);
                        eprintln!("Main Folder {} Added.", path);
                        if let Some(sub_path) = entry.get(i+1..entry.len()) {
                            new_entry.add_entry(sub_path, file_type);
                            eprintln!("Se pudo?");
                        }
                        eprintln!("In the Main Folder {}.", path);
                        self.add_folder(new_entry);

                    } else {
                        eprintln!("The subdir cannot be get");
                    }
                    return;
                }
            }
        }

        if let Some(path) = entry.get(0..entry.len()) {
            if let EntryType::Folder = file_type {
                eprintln!("Main Folder {} Added withouth follow.", path);
                self.add_folder(Folder::new(path))
            } else {
                eprintln!("File {} added.", path);
                self.add_file(path);
            }
        } else {
            eprintln!("No se pudo?");
        }

    }

    
    fn strace(&self, indent: usize) {
        let indent_char = " ".repeat(indent);
        println!("{}Folder: {}", " ".repeat(indent - 1) + "└┬", self.name);
        let mut i = 0;
        for entry in &self.content {
            match entry {
                Entry::File(file_name) => {
                    if i + 1 < self.content.len() {
                        println!("{} ├file: {}", indent_char, file_name);
                    } else {
                        println!("{} └file: {}", indent_char, file_name);
                    }
                },
                Entry::Folder(folder) => folder.strace(indent + 1),
            }
            i += 1;
        }
    }

    fn print(&self) {
        self.strace(1);
    }
}

struct Cursor {
    x: u16,
    y: u16
}

struct Window<'a> {
    root: Folder,
    current: Vec<Folder>,
    width: u16,
    height: u16,
    scroll_x: u16,
    scroll_y: u16,
    cursor: Cursor,
    path: String,
    writer: *mut StdoutLock<'a>
}

impl<'a> Window<'a> {
    fn new(width: u16, height: u16, stdout: *mut StdoutLock<'a>) -> Self {
        Self {
            root: Folder::new(""),
            current: vec![Folder::new("")],
            width, 
            height,
            scroll_x: 0,
            scroll_y: 0,
            cursor: Cursor { x: 1, y: 4 },
            path: String::new(),
            writer: stdout
        }
    }

    fn assign_root(&mut self, folder: Folder) {
        self.root = folder.clone();
        self.current = vec![folder];
    }

    fn get_current(&self) -> &Folder {
        &self.current[self.current.len() - 1]
    }
    
    fn set_current(&mut self, folder: Folder) {
        self.current.push(folder);
    }

    fn back_current(&mut self) {
        if self.current.len() > 1 {
            self.current.pop().unwrap();
        }
    }

    fn assign_path(&mut self, path: String) {
        self.path = path;
    }

    fn get_writer(&self) -> &mut StdoutLock<'a> {
        unsafe {
            &mut (*self.writer)
        }
    }



    fn get_path(&self) -> String {
        self.path.clone()
    } 

    fn move_up(&mut self) -> bool {
        if self.cursor.y > 4 {
            self.cursor.y -= 1;
        } else if self.scroll_y > 9 {
            self.scroll_y -= 10;
            return true;
        }
        false
    }

    fn move_down(&mut self) -> bool {
        if self.cursor.y < self.height - 2 {
            self.cursor.y += 1;
        } else {
            self.scroll_y += 10;
            return true;
        }
        false
    }

    fn move_left(&mut self) -> bool {
        if self.cursor.x > 1 {
            self.cursor.x -= 1;
        }
        false
    }

    fn move_right(&mut self) -> bool{
        if self.cursor.x < self.width - 2 {
            self.cursor.x += 1;
        }
        false
    }

    fn set_cursor(&mut self, x: u16, y: u16) {
        self.cursor.x = x;
        self.cursor.y = y;
    }
}

fn get_root(output: String) -> Folder {
    let mut root = Folder::new(".");

    let start_point: usize = output.find("   Date      Time    Attr         Size   Compressed  Name\n------------------- ----- ------------ ------------  ------------------------\n").expect("The content isn't be found") + "   Date      Time    Attr         Size   Compressed  Name\n------------------- ----- ------------ ------------  ------------------------\n".len();
    let clean_output = &output[start_point..];
    let lines: Vec<&str> = clean_output.split("\n").collect();

    for line in lines {
        if &line[20..25].to_string() == "-----" {
            break;
        }
        if &line[20..25].to_string() == "D...." {
            root.add_entry(&line[53..].to_string(), &EntryType::Folder);
        } else {
            root.add_entry(&line[53..].to_string(), &EntryType::File);
        }
        eprintln!("Name: {}, type: {}", &line[53..], &line[20..25]);
    }

    root
}

fn get_path(output: String) -> String {
    let path_start = output.find("Path = ").expect("No path") + 7;
    let path_end = output.find("\nType = ").expect("No path end");

    if let Some(path) = output.get(path_start..path_end) {
        String::from(path)
    } else {
        String::from("Unreacheable")
    }
}

/*trait FromDif {
    fn write_str(&mut self, content: &str);
    fn write_string(&mut self, content: String);
}

impl FromDif for Stdout {
    fn write_str(&mut self, content: &str) {
        self.write(content.as_bytes()).unwrap();
    }

    fn write_string(&mut self, content: String) {
    }
}*/

fn print_header(win: &Window) {
    let fill_all_block = "─".repeat(usize::from(win.width) - 2);
    let stdout = unsafe { &mut (*win.writer) };
    let path = win.get_path();

    stdout.queue(MoveTo(0, 0)).unwrap();
    stdout.write(("┌".to_string() + fill_all_block.as_str() + "┐").as_bytes()).unwrap();

    stdout.queue(MoveTo(0, 1)).unwrap();
    stdout.write("│".as_bytes()).unwrap();
    if path.len() > (win.width - 2).into() {
        stdout.write(&path.as_bytes()[0..path.len() - 3]).unwrap();
        stdout.write("...".as_bytes()).unwrap();
    } else {
        stdout.write(path.as_bytes()).unwrap();
    }

    stdout.queue(MoveTo(win.width - 1, 1)).unwrap();
    stdout.write("│".as_bytes()).unwrap();

    stdout.queue(MoveTo(0, 2)).unwrap();
    stdout.write(("└".to_string() + fill_all_block.as_str() + "┘").as_bytes()).unwrap();

    // stdout.queue(MoveTo(0, win.height - 1)).unwrap();
    // stdout.write_fmt(format_args!("{}:{}", win.cursor.x, win.cursor.y)).unwrap();
}

fn print_menu(win: &Window) {
    let fill_all_block = "─".repeat(usize::from(win.width) - 2);
    let stdout = unsafe { &mut (*win.writer) };

    stdout.queue(MoveTo(0, 3)).unwrap();
    stdout.write(("┌".to_string() + fill_all_block.as_str() + "┐").as_bytes()).unwrap();

    for i in 4..win.height {
        stdout.queue(MoveTo(0, i)).unwrap();
        stdout.queue(terminal::Clear(ClearType::CurrentLine)).unwrap();
        stdout.write("│".as_bytes()).unwrap();

        if win.get_current().content.len() > (i - 4 + win.scroll_y).into() {
            let entry = &win.get_current().content[usize::from(i - 4 + win.scroll_y)];
            let _ = match entry {
                Entry::File(file_name) => {
                    stdout.write("--- ".as_bytes()).unwrap();
                    stdout.write(file_name.as_bytes()).unwrap()
                },
                Entry::Folder(folder) => {
                    stdout.write("[+] ".as_bytes()).unwrap();
                    stdout.write(folder.name.as_bytes()).unwrap()
                },
            };
        }

        stdout.queue(MoveTo(win.width - 1, i)).unwrap();
        stdout.write("│".as_bytes()).unwrap();
    }

    stdout.queue(MoveTo(0, win.height - 1)).unwrap();
    stdout.write(("└".to_string() + fill_all_block.as_str() + "┘").as_bytes()).unwrap();
}

fn print_lines(win: &mut Window, lines: &Vec<&str>) {
    let stdout = unsafe { &mut (*win.writer) };

    MoveTo(0, 0);
    stdout.flush().unwrap();

    for i in usize::from(win.scroll_y)..usize::from(win.scroll_y + win.height) {
        if i >= lines.len() {
            break;
        }
        println!("\r\x1b[K{}", lines[i]);
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        eprintln!("Usage:\n\t{} {{7zip file}}", &args[0]);
        exit(-1);
    }

    let mut stdout = stdout().lock();
    stdout.queue(terminal::EnterAlternateScreen).unwrap();
    //stdout.queue(terminal::DisableLineWrap).unwrap();
    stdout.queue(terminal::EndSynchronizedUpdate).unwrap();
    terminal::enable_raw_mode().expect("Error al abrir la patalla");
    stdout.flush().unwrap();

    let (width, height) = terminal::size().unwrap();

    let mut win = Window::new(width, height, &mut stdout);

    let res = Command::new("7z")
        .args(vec!["l", &args[1]])
        .output()
        .expect("Ubo un error al ejecutar el commando!");
    let output = String::from_utf8(res.stdout).expect("No se pudo convertir la salida a texto");
    win.assign_root(get_root(output.clone()));
    eprintln!("El resultado es:\n{}", output);

    let path = get_path(output.clone());
    let binding = output.clone();
    let lines: Vec<&str> = binding.split("\n").collect();
    let mut mouse_update = false;
    win.assign_path(path);
    win.assign_root(get_root(output.clone()));
    print_header(&mut win);
    print_menu(&mut win);

    'mainLoop:
    loop {
        while event::poll(Duration::ZERO).unwrap() {
            match event::read().unwrap() {
                Event::Key(ev) => {
                    match ev.code {
                        KeyCode::Esc => break 'mainLoop,
                        KeyCode::Up => {
                            if win.move_up() {
                                print_menu(&mut win);
                            }
                            mouse_update = true;
                        },
                        KeyCode::Down => {
                            if win.move_down() {
                                print_menu(&mut win);
                            }
                            mouse_update = true;
                        },
                        KeyCode::Right => {
                            if win.move_right() {
                                print_menu(&mut win);
                            }
                            mouse_update = true;
                        },
                        KeyCode::Left => {
                            if win.move_left() {
                                print_menu(&mut win);
                            }
                            mouse_update = true;
                        },
                        KeyCode::Enter => {
                            if usize::from(win.cursor.y - 4) < win.get_current().content.len() {
                                if let Entry::Folder(dir) = &win.get_current().content[usize::from(win.cursor.y - 4)] {
                                    win.set_current(dir.clone());
                                    mouse_update = true;
                                    print_menu(&mut win);
                                }
                            }
                        },
                        KeyCode::Backspace => {
                            win.back_current();
                            print_menu(&mut win);
                        },
                        //KeyCode::Tab => print_lines(&mut win, &lines),
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        if mouse_update {
            let new_y: u16 = (win.get_current().content.len() + 3).try_into().unwrap();
            if win.cursor.y > new_y && new_y != 3 {
                win.set_cursor(win.cursor.x, new_y);
            }
            stdout.queue(MoveTo(win.cursor.x, win.cursor.y)).unwrap();
        }

        stdout.flush().unwrap();

        thread::sleep(Duration::from_millis(30));
    }

    terminal::disable_raw_mode().unwrap();
    stdout.queue(terminal::LeaveAlternateScreen).unwrap();
}
