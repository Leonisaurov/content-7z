use std::process::{Command, exit};
use std::env::args;

use crossterm::{
    self, terminal::{self, Clear, ClearType}, cursor::MoveTo,
    QueueableCommand,
    event::{self, Event, KeyCode}
};
use std::{time::Duration, io::{stdout, Write}, thread};
use content_7z::{
    files::{folder::Folder, entry::Entry},
    window::window::Window
};

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
    let path = win.get_path() + win.plain_current().as_str();

    stdout.queue(MoveTo(0, 0)).unwrap();
    stdout.write(("┌".to_string() + fill_all_block.as_str() + "┐").as_bytes()).unwrap();

    stdout.queue(MoveTo(0, 1)).unwrap();
    stdout.queue(Clear(ClearType::CurrentLine)).unwrap();
    stdout.write("│".as_bytes()).unwrap();
    if path.len() > (win.width - 2).into() {
        stdout.write("...".as_bytes()).unwrap();
        stdout.write(&path.as_bytes()[path.len() - usize::from(win.width - 5)..path.len()]).unwrap();
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

fn show_dialog(win: &mut Window, label: &str) {
    let stdout = unsafe { &mut (*win.writer) };
    let x: u16 = win.width / 2 - label.len() as u16 / 2;
    let y: u16 = win.height / 2 - 1;

    let fill_all_block = "─".repeat(label.len());

    stdout.queue(MoveTo(x, y)).unwrap();
    stdout.write("┌".as_bytes()).unwrap();
    stdout.write(fill_all_block.as_bytes()).unwrap();
    stdout.write("┐".as_bytes()).unwrap();

    stdout.queue(MoveTo(x, y + 1)).unwrap();
    stdout.write("│".as_bytes()).unwrap();
    stdout.write(label.as_bytes()).unwrap();
    stdout.write("│".as_bytes()).unwrap();

    stdout.queue(MoveTo(x, y + 2)).unwrap();
    stdout.write("└".as_bytes()).unwrap();
    stdout.write(fill_all_block.as_bytes()).unwrap();
    stdout.write("┘".as_bytes()).unwrap();

    win.on_dialog = true;

    win.cursor.x = win.width / 2;
    win.cursor.y = win.height / 2;

    win.cursor.need_update = true;
}

#[allow(dead_code)]
fn close_dialog(win: &mut Window) {
    win.on_dialog = false;
    print_header(win);
    print_menu(win);
}

/*
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
}*/

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
        .output().expect("Error al obtener los datos del archivo!");
    if String::from_utf8(res.stderr.clone()).unwrap() != "" {
        terminal::disable_raw_mode().unwrap();
        stdout.queue(terminal::LeaveAlternateScreen).unwrap();
        stdout.flush().unwrap();
        eprintln!("Process Error: {}", String::from_utf8(res.stderr).unwrap());
        exit(-1);
    }

    let output = String::from_utf8(res.stdout).expect("No se pudo convertir la salida a texto");
    win.assign_root(Folder::get_root(output.clone()));
    //eprintln!("El resultado es:\n{}", output);

    let path = get_path(output.clone());
    //let binding = output.clone();
    //let lines: Vec<&str> = binding.split("\n").collect();
    win.assign_path(path);
    print_header(&mut win);
    print_menu(&mut win);
    stdout.queue(MoveTo(1, 4)).unwrap();

    'mainLoop:
    loop {
        while event::poll(Duration::ZERO).unwrap() {
            match event::read().unwrap() {
                Event::Key(ev) => {
                    match ev.code {
                        KeyCode::Esc => break 'mainLoop,
                        KeyCode::Up => win.move_up(),
                        KeyCode::Down => win.move_down(),
                        KeyCode::Right => win.move_right(),
                        KeyCode::Left => win.move_left(),

                        KeyCode::Char('s') => show_dialog(&mut win, "Hola, como estas?"),

                        KeyCode::Backspace => win.back_current(),
                        KeyCode::Enter => {
                            if usize::from(win.cursor.y - 4 + win.scroll_y) < win.get_current().content.len() {
                                match &win.get_current().content[usize::from(win.cursor.y - 4 + win.scroll_y)] {
                                    Entry::Folder(dir) => win.set_current(dir.clone()),
                                    Entry::File(file_name) => {
                                        let path = win.plain_current() + file_name;
                                        show_dialog(&mut win, path.as_str());
                                    },
                                }
                            }
                        },
                        //KeyCode::Tab => print_lines(&mut win, &lines),
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        if win.scroll_change {
            win.scroll_change = false;
            print_menu(&mut win);
        }

        if win.cursor.need_update {
            win.cursor.need_update = false;
            let new_y: u16 = (win.get_current().content.len() + 3).try_into().unwrap();
            if !win.on_dialog && win.cursor.y > new_y && new_y != 3 {
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
