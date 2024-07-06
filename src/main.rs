use crossterm::{
    self, terminal::{self, Clear, ClearType}, cursor::MoveTo,
    QueueableCommand,
    event::{self, Event, KeyCode}
};

use std::{
    time::Duration, io::{stdout, Write}, thread,
    process::{exit, Command, Stdio}, env,
};

use content_7z::{
    files::entry::Entry,
    window::{
        window::Window,
        scheme::NOCOLOR,
        handler::{Handler, HandleSituatonType, NormalHandler},
    },
    config
};

use content_7z::zip_manager::manager::ZipManager;
use std::path::PathBuf;

fn print_header(win: &Window) {
    let fill_all_block = "─".repeat(usize::from(win.width) - 2);
    let stdout = unsafe { &mut (*win.writer) };
    let path = win.get_path() + win.plain_current().as_str();

    stdout.queue(MoveTo(0, 0)).unwrap();
    stdout.write(win.scheme.background_color.repr.as_bytes()).unwrap();
    stdout.write(win.scheme.border_color.repr.as_bytes()).unwrap();
    stdout.write(("┌".to_string() + fill_all_block.as_str() + "┐").as_bytes()).unwrap();

    stdout.queue(MoveTo(0, 1)).unwrap();
    stdout.queue(Clear(ClearType::CurrentLine)).unwrap();
    stdout.write(win.scheme.border_color.repr.as_bytes()).unwrap();
    stdout.write("│".as_bytes()).unwrap();
    if path.len() > (win.width - 2).into() {
        stdout.write(win.scheme.text_color.repr.as_bytes()).unwrap();
        stdout.write("...".as_bytes()).unwrap();
        stdout.write(&path.as_bytes()[path.len() - usize::from(win.width - 5)..path.len()]).unwrap();
    } else {
        stdout.write(win.scheme.text_color.repr.as_bytes()).unwrap();
        stdout.write(path.as_bytes()).unwrap();
    }

    stdout.queue(MoveTo(win.width - 1, 1)).unwrap();
    stdout.write(win.scheme.border_color.repr.as_bytes()).unwrap();
    stdout.write("│".as_bytes()).unwrap();

    stdout.queue(MoveTo(0, 2)).unwrap();
    stdout.write(win.scheme.border_color.repr.as_bytes()).unwrap();
    stdout.write(("└".to_string() + fill_all_block.as_str() + "┘").as_bytes()).unwrap();
    stdout.write(NOCOLOR).unwrap();
}

fn print_menu(win: &Window) {
    let fill_all_block = "─".repeat(usize::from(win.width) - 2);
    let stdout = unsafe { &mut (*win.writer) };

    stdout.queue(MoveTo(0, 3)).unwrap();
    stdout.write(win.scheme.background_color.repr.as_bytes()).unwrap();
    stdout.write(win.scheme.border_color.repr.as_bytes()).unwrap();
    stdout.write(("┌".to_string() + fill_all_block.as_str() + "┐").as_bytes()).unwrap();

    for i in 4..win.height {
        stdout.queue(MoveTo(0, i)).unwrap();
        stdout.queue(terminal::Clear(ClearType::CurrentLine)).unwrap();
        stdout.write(win.scheme.border_color.repr.as_bytes()).unwrap();
        stdout.write("│".as_bytes()).unwrap();

        if win.get_current().content.len() > (i - 4 + win.scroll_y).into() {
            let entry = &win.get_current().content[usize::from(i - 4 + win.scroll_y)];
            let _ = match entry {
                Entry::File(file_name) => {
                    stdout.write(NOCOLOR).unwrap();
                    stdout.write(win.scheme.file_bullet_color.repr.as_bytes()).unwrap();
                    stdout.write(win.scheme.file_bullet.as_bytes()).unwrap();
                    stdout.write(NOCOLOR).unwrap();
                    stdout.write(win.scheme.text_color.repr.as_bytes()).unwrap();
                    stdout.write(file_name.as_bytes()).unwrap()
                },
                Entry::Folder(folder) => {
                    stdout.write(NOCOLOR).unwrap();
                    stdout.write(win.scheme.folder_bullet_color.repr.as_bytes()).unwrap();
                    stdout.write(win.scheme.folder_bullet.as_bytes()).unwrap();
                    stdout.write(NOCOLOR).unwrap();
                    stdout.write(win.scheme.text_color.repr.as_bytes()).unwrap();
                    stdout.write(folder.name.as_bytes()).unwrap()
                },
            };
        }

        stdout.queue(MoveTo(win.width - 1, i)).unwrap();
        stdout.write(win.scheme.border_color.repr.as_bytes()).unwrap();
        stdout.write("│".as_bytes()).unwrap();
    }

    stdout.queue(MoveTo(0, win.height - 1)).unwrap();
    stdout.write(win.scheme.border_color.repr.as_bytes()).unwrap();
    stdout.write(("└".to_string() + fill_all_block.as_str() + "┘").as_bytes()).unwrap();
    stdout.write(NOCOLOR).unwrap();
}

fn show_dialog(win: &mut Window, text: String) {
    let stdout = unsafe { &mut (*win.writer) };

    let lines_iter = text.split("\n");
    let mut lines = vec![];
    let mut max_length = 0;

    for line in lines_iter {
        if line.len() > usize::from(win.width - 8) {
            max_length = win.width - 8;
            let mut index: usize = 0;
            loop {
                if line.len() < usize::from(max_length) + index {
                    lines.push(&line[index..]);
                    break;
                }

                lines.push(&line[index..index + usize::from(max_length)]);
                index += usize::from(max_length);
            }
        } else if line.len() > usize::from(max_length) {
            max_length = line.len() as u16;
            lines.push(line);
        }
    }

    let x: u16 = win.width / 2 - (max_length + 2) / 2;
    let y: u16 = win.height / 2 - (lines.len() + 2) as u16 / 2;

    let fill_all_block = "─".repeat(usize::from(max_length));

    stdout.queue(MoveTo(x, y)).unwrap();
    stdout.write("┌".as_bytes()).unwrap();
    stdout.write(fill_all_block.as_bytes()).unwrap();
    stdout.write("┐".as_bytes()).unwrap();

    for (index, line) in lines.iter().enumerate() {
        stdout.queue(MoveTo(x, y + 1 + index as u16)).unwrap();
        stdout.write("│".as_bytes()).unwrap();
        stdout.write(line.as_bytes()).unwrap();
        stdout.queue(MoveTo(x + max_length + 1, y + 1 + index as u16)).unwrap();
        stdout.write("│".as_bytes()).unwrap();
    }

    stdout.queue(MoveTo(x, y + 1 + lines.len() as u16)).unwrap();
    stdout.write("└".as_bytes()).unwrap();
    stdout.write(fill_all_block.as_bytes()).unwrap();
    stdout.write("┘".as_bytes()).unwrap();

    win.on_dialog = true;

    stdout.queue(MoveTo(win.width / 2, win.height / 2)).unwrap();
}

const HELP_ANSWER_LABEL: &str = "y(es),n(o)";

fn show_confirm_dialog<T: Handler + 'static>(win: &mut Window, label: String, handler: T) {
    let stdout = unsafe { &mut (*win.writer) };

    let max = if label.len() < 10 {
        HELP_ANSWER_LABEL.len()
    } else {
        label.len()
    };

    let x: u16 = win.width / 2 - max as u16 / 2;
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
    stdout.queue(MoveTo(x + (max / 2) as u16 - (HELP_ANSWER_LABEL.len() / 2) as u16, y + 2)).unwrap();
    stdout.write(HELP_ANSWER_LABEL.as_bytes()).unwrap();

    win.on_dialog = true;

    stdout.queue(MoveTo(win.width / 2, win.height / 2)).unwrap();
    win.handler = Some(Box::new(handler));
}

fn close_dialog(win: &mut Window) {
    print_header(win);
    print_menu(win);

    win.cursor.need_update = true;
    win.on_dialog = false;
}

fn get_temp_dir(win: &mut Window) -> String {
    if win.tmp_dir == "" {
        let output = Command::new("mktemp")
            .args(["-d", "--tmpdir", "content_7z.XXX"])
            .output()
            .expect("Getting tmp dir fail.");
        let tmp_dir = String::from_utf8(output.stdout).expect("Cannot get the tmp dir.");
        win.tmp_dir = String::from(&tmp_dir[..tmp_dir.len() - 1]);
    }
    win.tmp_dir.clone() + "/" + win.get_path().as_str()
}

fn open_editor(win: &mut Window, file: PathBuf) {
    let stdout = unsafe {
        &mut (*win.writer)
    };

    stdout.queue(Clear(ClearType::Purge)).unwrap();
    stdout.flush().unwrap();

    // Opening tmp_dir + path
    let status = Command::new(win.scheme.editor.clone())
        .arg(file.to_str().unwrap())
        .status()
        .expect("Couldnt open the editor");

    if status.code().expect("Cannot stablish connection with the editor") != 0 {
        // TODO
    }

    let stdout = unsafe {
        &mut (*win.writer)
    };

    win.open_window();
    stdout.queue(Clear(ClearType::Purge)).unwrap();
    stdout.flush().unwrap();

    print_menu(win);
    print_header(win);

    stdout.queue(MoveTo(win.cursor.x, win.cursor.y)).unwrap();
    stdout.flush().unwrap();
}

fn extract_an_open_file(win: &mut Window, tmp_dir: String, file_name: String, file: PathBuf, overwrite: bool) {
    let path = if let Some(path) = file_name.rfind("/") {
        vec![&file_name[..path + 1], &file_name[path + 1..]]
    } else {
        vec!["/", file_name.as_str()]
    };

    // Making the path directories
    let mkdir_status = Command::new("mkdir")
        .args(["-p", (tmp_dir.clone() + path[0]).as_str()])
        .stdout(Stdio::null())
        .status().expect("Cannot communicate with the terminal");

    if mkdir_status.code().expect("Cannot communicate with the 'mkdir' command") != 0 {
        // TODO
        return;
    }

    // Extracting the file to: tmp_dir + path[0]
    let win_path = win.get_path();
    let output_path = format!("-o{}/{}", tmp_dir, path[0]);

    let mut extractor_args = vec!["e", win_path.as_str(), &file_name[1..], output_path.as_str()];
    if overwrite {
        extractor_args.push("-y");
    }
    let extract_status = Command::new("7z")
        .args(extractor_args)
        .stdout(Stdio::null())
        .status().expect("Cannot execute the extractor.");

    if extract_status.code().expect("Cannot extract the file from the compress file.") != 0 {
        // TODO
        return
    }

    open_editor(win, file);
}

fn open_file(win: &mut Window, file_name: String) {
    // Getting the tmp dir for this session
    let tmp_dir = get_temp_dir(win);
    // Getting a file refence
    let file = PathBuf::from(tmp_dir.clone() + "/" + file_name.as_str());
    if !win.scheme.always_overwrite && file.exists() {
        let job = NormalHandler::new(|win, situation, data| {
            if let HandleSituatonType::SUCESS = situation {
                extract_an_open_file(win, data.2.clone(), data.0.clone(), data.1.clone(), true);
            } else {
                open_editor(win, data.1.clone());
            }
        }, (file_name, file, tmp_dir));
        show_confirm_dialog(win, String::from("File already extracted, pressent on cache, extract it again?"), job);
        return;
    }

    extract_an_open_file(win, tmp_dir, file_name, file, false);
} 

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage:\n\t{} {{7zip file}}", &args[0]);
        exit(-1);
    }

    let mut stdout = stdout().lock();

    let mut win = Window::new(&mut stdout, config::load());
    let manager = ZipManager::process(&args[1]);
    if manager.err != "" {
        eprintln!("Process Error: {}", manager.err);
        exit(-1);
    }

    win.assing_manager(manager);

    print_header(&mut win);
    print_menu(&mut win);
    stdout.queue(MoveTo(1, 4)).unwrap();

    'mainLoop:
    loop {
        while event::poll(Duration::ZERO).unwrap() {
            if win.on_dialog {
                if let Event::Key(key) = event::read().unwrap() {
                    close_dialog(&mut win);
                    match key.code {
                        KeyCode::Char('y') => win.run_job(HandleSituatonType::SUCESS),
                        KeyCode::Char('n') => win.run_job(HandleSituatonType::DENIED),
                        _ => win.run_job(HandleSituatonType::UNDECIDED),
                    }
                }
                break;
            }
            match event::read().unwrap() {
                Event::Key(ev) => {
                    match ev.code {
                        KeyCode::Esc | KeyCode::Char('q') => break 'mainLoop,
                        KeyCode::Up => win.move_up(),
                        KeyCode::Down => win.move_down(),
                        KeyCode::Right => win.move_right(),
                        KeyCode::Left => win.move_left(),
                        KeyCode::Char('t') => show_dialog(&mut win, ("Hello".repeat(20) + "\n").repeat(3)),
                        KeyCode::Char('p') => {
                            let path = win.plain_current();
                            show_dialog(&mut win, path);
                            continue 'mainLoop;
                        },
                        KeyCode::Char('o') => {
                            if usize::from(win.cursor.y - 4 + win.scroll_y) < win.get_current().content.len() {
                                if let Entry::File(file_name) = &win.get_current().content[usize::from(win.cursor.y - 4 + win.scroll_y)] {
                                    let path = win.plain_current() + "/" + file_name;
                                    let message = format!("Open '{}'?", path);

                                    let job = NormalHandler::new(|win, situation, file_name| {
                                        if let HandleSituatonType::SUCESS = situation {
                                            open_file(win, file_name.clone());
                                        }
                                    }, path);

                                    show_confirm_dialog(&mut win, message, job);
                                    continue 'mainLoop;
                                }
                            }
                        }
                        KeyCode::Backspace => win.back_current(),
                        KeyCode::Enter => {
                            if usize::from(win.cursor.y - 4 + win.scroll_y) < win.get_current().content.len() {
                                if let Entry::Folder(dir) = &win.get_current().content[usize::from(win.cursor.y - 4 + win.scroll_y)] {
                                    win.set_current(dir.clone());
                                }
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        if win.scroll_change {
            win.scroll_change = false;
            print_menu(&mut win);
            stdout.queue(MoveTo(win.cursor.x, win.cursor.y)).unwrap();
        }

        if win.path_change {
            win.path_change = false;
            print_header(&win);
            stdout.queue(MoveTo(win.cursor.x, win.cursor.y)).unwrap();
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
}
