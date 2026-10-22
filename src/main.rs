use crossterm::{
    self, cursor::MoveTo, event::{self, Event, KeyCode}, terminal::{self, Clear, ClearType}, QueueableCommand
};

use std::{
    env, fmt::{self}, io::{stdout, Write}, process::{exit, Command, Stdio}, thread, time::Duration
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

enum Color {
    RED,
    REMOVE
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Color::RED => "\x1b[31m",
            Color::REMOVE => "\x1b[0m",
        };
        write!(f, "{}", s)
    }
}

fn print_header(win: &Window) {
    let fill_all_block = "─".repeat(usize::from(win.width.saturating_sub(2)));
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
    if path.len() > usize::from(win.width.saturating_sub(2)) {
        stdout.write(win.scheme.text_color.repr.as_bytes()).unwrap();
        stdout.write("...".as_bytes()).unwrap();
        let start = path.len().saturating_sub(usize::from(win.width.saturating_sub(5)));
        stdout.write(&path.as_bytes()[start..path.len()]).unwrap();
    } else {
        stdout.write(win.scheme.text_color.repr.as_bytes()).unwrap();
        stdout.write(path.as_bytes()).unwrap();
    }

    stdout.queue(MoveTo(win.width.saturating_sub(1), 1)).unwrap();
    stdout.write(win.scheme.border_color.repr.as_bytes()).unwrap();
    stdout.write("│".as_bytes()).unwrap();

    stdout.queue(MoveTo(0, 2)).unwrap();
    stdout.write(win.scheme.border_color.repr.as_bytes()).unwrap();
    stdout.write(("└".to_string() + fill_all_block.as_str() + "┘").as_bytes()).unwrap();
    stdout.write(NOCOLOR).unwrap();
}

fn print_menu(win: &Window) {
    let fill_all_block = "─".repeat(usize::from(win.width.saturating_sub(2)));
    let stdout = unsafe { &mut (*win.writer) };

    stdout.queue(MoveTo(0, 3)).unwrap();
    stdout.write(win.scheme.background_color.repr.as_bytes()).unwrap();
    stdout.write(win.scheme.border_color.repr.as_bytes()).unwrap();
    stdout.write(("┌".to_string() + fill_all_block.as_str() + "┐").as_bytes()).unwrap();

    let selected_index = usize::from(win.cursor.y.saturating_sub(4)) + usize::from(win.scroll_y);

    for i in 4..win.height {
        stdout.queue(MoveTo(0, i)).unwrap();
        stdout.queue(terminal::Clear(ClearType::CurrentLine)).unwrap();
        stdout.write(win.scheme.border_color.repr.as_bytes()).unwrap();
        stdout.write("│".as_bytes()).unwrap();

        let entry_index = usize::from(i - 4) + usize::from(win.scroll_y);
        if win.get_current().content.len() > entry_index {
            let entry = &win.get_current().content[entry_index];
            let is_selected = !win.on_dialog && entry_index == selected_index;
            if is_selected {
                stdout.write(b"\x1b[7m").unwrap();
            }
            let _ = match entry {
                Entry::File(file_name) => {
                    stdout.write(NOCOLOR).unwrap();
                    stdout.write(win.scheme.background_color.repr.as_bytes()).unwrap();
                    stdout.write(win.scheme.file_bullet_color.repr.as_bytes()).unwrap();
                    stdout.write(win.scheme.file_bullet.as_bytes()).unwrap();
                    stdout.write(NOCOLOR).unwrap();
                    if is_selected {
                        stdout.write(b"\x1b[7m").unwrap();
                    }
                    stdout.write(win.scheme.background_color.repr.as_bytes()).unwrap();
                    stdout.write(win.scheme.text_color.repr.as_bytes()).unwrap();
                    stdout.write(file_name.as_bytes()).unwrap()
                },
                Entry::Folder(folder) => {
                    stdout.write(NOCOLOR).unwrap();
                    stdout.write(win.scheme.background_color.repr.as_bytes()).unwrap();
                    stdout.write(win.scheme.folder_bullet_color.repr.as_bytes()).unwrap();
                    stdout.write(win.scheme.folder_bullet.as_bytes()).unwrap();
                    stdout.write(NOCOLOR).unwrap();
                    if is_selected {
                        stdout.write(b"\x1b[7m").unwrap();
                    }
                    stdout.write(win.scheme.background_color.repr.as_bytes()).unwrap();
                    stdout.write(win.scheme.text_color.repr.as_bytes()).unwrap();
                    stdout.write(folder.name.as_bytes()).unwrap()
                },
            };
            if is_selected {
                stdout.write(NOCOLOR).unwrap();
                stdout.write(win.scheme.background_color.repr.as_bytes()).unwrap();
            }
        }

        stdout.queue(MoveTo(win.width.saturating_sub(1), i)).unwrap();
        stdout.write(win.scheme.border_color.repr.as_bytes()).unwrap();
        stdout.write("│".as_bytes()).unwrap();
    }

    stdout.queue(MoveTo(0, win.height.saturating_sub(1))).unwrap();
    stdout.write(win.scheme.border_color.repr.as_bytes()).unwrap();
    stdout.write(("└".to_string() + fill_all_block.as_str() + "┘").as_bytes()).unwrap();
    stdout.write(NOCOLOR).unwrap();
}

fn show_dialog_raw(win: &mut Window, text: String, helper: Option<&str>) {
    let stdout = unsafe { &mut (*win.writer) };

    let lines_iter = text.split("\n");
    let mut lines = vec![];
    let mut max_length = 0u16;

    'scanner:
    for line in lines_iter {
        if line.len() > usize::from(win.width.saturating_sub(8)) {
            max_length = win.width.saturating_sub(8);
            let mut index: usize = 0;
            loop {
                if line.len() < usize::from(max_length) + index {
                    lines.push(&line[index..]);
                    continue 'scanner;
                }

                lines.push(&line[index..index + usize::from(max_length)]);
                index += usize::from(max_length);
            }
        }

        if line.len() as u16 > max_length {
            max_length = line.len() as u16;
        }
        lines.push(line);
    }

    let mut helper_label = vec![];

    if let Some(label) = helper {
        'scanner:
        for line in label.split('\n') {
            if line.len() > usize::from(win.width.saturating_sub(8)) {
                max_length = win.width.saturating_sub(8);
                let mut index: usize = 0;
                loop {
                    if line.len() < usize::from(max_length) + index {
                        helper_label.push(&line[index..]);
                        continue 'scanner;
                    }

                    helper_label.push(&line[index..index + usize::from(max_length)]);
                    index += usize::from(max_length);
                }
            }

            if line.len() as u16 > max_length {
                max_length = line.len() as u16;
            }
            helper_label.push(line);
        }
    }

    let helper_label_increment = if helper_label.len() < 2 {
        0
    } else {
        helper_label.len() - 1
    };

    let x: u16 = win.width / 2 - (max_length + 2) / 2;
    let y: u16 = win.height / 2 - (lines.len() + helper_label_increment + 2) as u16 / 2;

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
    if helper_label.len() < 2 {
        stdout.write("└".as_bytes()).unwrap();
        stdout.write(fill_all_block.as_bytes()).unwrap();
        stdout.write("┘".as_bytes()).unwrap();

        if helper_label.len() == 1 {
            stdout.queue(MoveTo(x + max_length / 2 - helper_label[0].len() as u16 / 2 + 1, y + 1 + lines.len() as u16)).unwrap();
            stdout.write(helper_label[0].as_bytes()).unwrap();
        }
    } else {
        stdout.write("├".as_bytes()).unwrap();
        stdout.write(fill_all_block.as_bytes()).unwrap();
        stdout.write("┤".as_bytes()).unwrap();

        stdout.queue(MoveTo(x, y + helper_label.len() as u16 + lines.len() as u16)).unwrap();
        stdout.write("└".as_bytes()).unwrap();
        stdout.write(fill_all_block.as_bytes()).unwrap();
        stdout.write("┘".as_bytes()).unwrap();

        for (index, label) in helper_label.iter().enumerate() {
            if index != 0 && index != helper_label.len() - 1 {
                stdout.queue(MoveTo(x, y + 1 + lines.len() as u16 + index as u16)).unwrap();
                stdout.write("│".as_bytes()).unwrap();
            }

            stdout.queue(MoveTo(x + max_length / 2 - label.len() as u16 / 2 + 1, y + 1 + lines.len() as u16 + index as u16)).unwrap();
            stdout.write(label.as_bytes()).unwrap();

            if index != 0 && index != helper_label.len() - 1 {
                stdout.queue(MoveTo(x + max_length + 1, y + 1 + lines.len() as u16 + index as u16)).unwrap();
                stdout.write("│".as_bytes()).unwrap();
            }
        }
    }

    win.on_dialog = true;

    if helper_label.len() == 0 {
        stdout.queue(MoveTo(win.width / 2, win.height / 2)).unwrap();
    } else {
        stdout.queue(MoveTo(x + max_length / 2 + 1, y + (lines.len() + helper_label.len() / 2) as u16 + 1)).unwrap();
    }
}

fn show_dialog(win: &mut Window, text: String) {
    show_dialog_raw(win, text, None);
}

fn show_multiple_choice_dialog<T: Handler + 'static>(win: &mut Window, quest: String, handler: T) {
    let helper = win.scheme.multi_choice_dialog_helper.clone();
    show_dialog_raw(win, quest, Some(helper.as_str()));

    win.handler = Some(Box::new(handler));
}

fn show_err_dialog(win: &mut Window, err: &str, exit: bool) {
    show_dialog_raw(win, String::from(err), None);
    if exit {
        let _ = terminal::disable_raw_mode();
        unsafe {
            let w = &mut (*win.writer);
            let _ = w.queue(terminal::LeaveAlternateScreen);
            let _ = w.flush();
        }
        std::process::exit(1);
    }
}

fn close_dialog(win: &mut Window) {
    print_header(win);
    print_menu(win);

    win.cursor.need_update = true;
    win.on_dialog = false;
}

fn get_temp_dir(win: &mut Window) -> String {
    if win.tmp_dir.is_empty() {
        let output = Command::new("mktemp")
            .args(["-d", "--tmpdir", "content_7z.XXX"])
            .output();
        match output {
            Ok(out) => {
                let tmp_dir = String::from_utf8_lossy(&out.stdout).to_string();
                win.tmp_dir = tmp_dir.trim().to_string();
            }
            Err(_) => {
                win.tmp_dir = std::env::temp_dir().to_string_lossy().to_string();
            }
        }
    }
    format!("{}/{}", win.tmp_dir, win.get_path())
}

fn open_editor(win: &mut Window, file: PathBuf) {
    let stdout = unsafe {
        &mut (*win.writer)
    };

    stdout.queue(Clear(ClearType::Purge)).unwrap();
    stdout.flush().unwrap();

    if win.scheme.editor.is_empty() {
        show_err_dialog(win, "Content-7z can not assumed any editor.\nDefine one in the config file:\n~/.config/content-7z.toml", false);
        return;
    }

    let status = Command::new(win.scheme.editor.clone())
        .arg(file.to_str().unwrap_or(""))
        .status();

    match status {
        Ok(s) => {
            if !s.success() {
                show_err_dialog(win, "Editor exited with an error.", false);
            }
        }
        Err(_) => {
            show_err_dialog(win, "Could not launch the editor.", false);
        }
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
    let path = if let Some(pos) = file_name.rfind('/') {
        vec![&file_name[..pos + 1], &file_name[pos + 1..]]
    } else {
        vec!["/", file_name.as_str()]
    };

    // Making the path directories
    let mkdir_status = Command::new("mkdir")
        .args(["-p", (tmp_dir.clone() + path[0]).as_str()])
        .stdout(Stdio::null())
        .status();

    match mkdir_status {
        Ok(s) if s.success() => {}
        _ => {
            show_err_dialog(win, "Cannot create extraction directory.", false);
            return;
        }
    }

    // Extracting the file to: tmp_dir + path[0]
    let win_path = win.get_path();

    let extract_status = if win_path.ends_with(".tar.gz") {
        let output_path = format!("{}/{}", tmp_dir, path[0]);
        let mut extractor_args = vec!["-xzf", win_path.as_str(), "-C", output_path.as_str(), &file_name[1..]];
        if !overwrite {
            extractor_args.push("--keep-old-files");
        }
        Command::new("tar")
            .args(extractor_args)
            .stdout(Stdio::null())
            .status()
    } else if win_path.ends_with(".tar") {
        let output_path = format!("{}/{}", tmp_dir, path[0]);
        let mut extractor_args = vec!["-xf", win_path.as_str(), "-C", output_path.as_str(), &file_name[1..]];
        if !overwrite {
            extractor_args.push("--keep-old-files");
        }
        Command::new("tar")
            .args(extractor_args)
            .stdout(Stdio::null())
            .status()
    } else if win_path.ends_with(".zip") {
        let output_path = format!("{}/{}", tmp_dir, path[0]);
        let mut extractor_args: Vec<String> = vec!["-q".to_string(), win_path.clone(), file_name[1..].to_string(), "-d".to_string(), output_path];
        if overwrite {
            extractor_args.insert(0, "-o".to_string());
        } else {
            extractor_args.insert(0, "-n".to_string());
        }
        if let Some(password) = win.get_file_password() {
            let pw = password.trim_start_matches("-p").to_string();
            extractor_args.push("-P".to_string());
            extractor_args.push(pw);
        }
        Command::new("unzip")
            .args(extractor_args)
            .stdout(Stdio::null())
            .status()
    } else {
        let output_path = format!("-o{}/{}", tmp_dir, path[0]);
        let mut extractor_args: Vec<String> = vec!["e".to_string(), win_path, file_name[1..].to_string(), output_path];
        if overwrite {
            extractor_args.push("-y".to_string());
        } else {
            extractor_args.push("-n".to_string());
        }

        if let Some(password) = win.get_file_password() {
            extractor_args.push(password);
        }

        Command::new("7z")
            .args(extractor_args)
            .stdout(Stdio::null())
            .status()
    };

    match extract_status {
        Ok(s) if s.success() => {}
        _ => {
            show_err_dialog(win, "Could not extract the file.", false);
            return;
        }
    }

    open_editor(win, file);
}

fn open_file(win: &mut Window, file_name: String) {
    let tmp_dir = get_temp_dir(win);
    let file = PathBuf::from(format!("{}/{}", tmp_dir, file_name));
    if !win.scheme.always_overwrite && file.exists() {
        let job = NormalHandler::new(|win, situation, data| {
            if let HandleSituatonType::SUCESS(_) = situation {
                extract_an_open_file(win, data.2.clone(), data.0.clone(), data.1.clone(), true);
            } else if let HandleSituatonType::DENIED = situation {
                open_editor(win, data.1.clone());
            }
        }, (file_name, file, tmp_dir));
        show_multiple_choice_dialog(win, String::from("File already extracted, present on cache.\nExtract it again?"), job);
        return;
    }

    extract_an_open_file(win, tmp_dir, file_name, file, false);
}

fn print_help(win: &mut Window) {
    let text = "Keys:\n\
↑/↓ navigate\n\
←/→ scroll\n\
Enter open folder\n\
Backspace/- go back\n\
o open file\n\
p show path\n\
h/? help\n\
q/Esc quit";
    show_dialog(win, String::from(text));
}

fn main() {
    // restore terminal on panic
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = terminal::disable_raw_mode();
        let mut out = stdout();
        let _ = out.queue(terminal::LeaveAlternateScreen);
        let _ = out.flush();
        default_hook(info);
    }));

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage:\n\t{} {{Compress file}} -p[password]", &args[0]);
        exit(-1);
    }

    let need_password = ZipManager::need_password(&args[1]);
    let password = if need_password {
        if args.len() != 3 {
            eprintln!("No password given, the file '{}' needs one to view the content.", &args[1]);
            exit(-2);
        }
        let password = &args[2];
        if !password.starts_with("-p") {
            eprintln!("Unknown argument '{}'", password);
            exit(-2);
        }
        Some(args[2].as_str())
    } else {
        None
    };

    let manager = ZipManager::process(&args[1], password);
    if manager.err != "" && manager.res_code != 0 {
        match manager.res_code {
            2 => eprintln!("{}The password provided is not the correct one.{}", Color::RED, Color::REMOVE),
            _ => eprintln!("Process Error: {}", manager.err),
        }
        exit(-1);
    }

    let mut stdout = stdout().lock();
    let mut win = Window::new(&mut stdout, config::load(), password);

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
                        KeyCode::Char('y') => win.run_job(HandleSituatonType::SUCESS(true)),
                        KeyCode::Enter => win.run_job(HandleSituatonType::SUCESS(false)),

                        KeyCode::Char('n') => win.run_job(HandleSituatonType::DENIED),

                        KeyCode::Char(ch) => win.run_job(HandleSituatonType::KEY(ch)),
                        _ => {},
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
                        KeyCode::Char('h') | KeyCode::Char('?') => {
                            print_help(&mut win);
                        },
                        KeyCode::Char('p') => {
                            let path = win.plain_current();
                            show_dialog(&mut win, path);
                        },
                        KeyCode::Char('o') => {
                            if usize::from(win.cursor.y.saturating_sub(4) + win.scroll_y) < win.get_current().content.len() {
                                if let Entry::File(file_name) = &win.get_current().content[usize::from(win.cursor.y.saturating_sub(4) + win.scroll_y)] {
                                    let path = win.plain_current() + "/" + file_name;
                                    let message = format!("Open '{}'?", path);

                                    let job = NormalHandler::new(|win, situation, file_name| {
                                        if let HandleSituatonType::SUCESS(direct) = situation {
                                            if direct {
                                                open_file(win, file_name.clone());
                                            }
                                        }
                                    }, path);

                                    show_multiple_choice_dialog(&mut win, message, job);
                                    continue 'mainLoop;
                                }
                            }
                        }
                        KeyCode::Backspace => win.back_current(),
                        KeyCode::Char('-') => win.back_current(),
                        KeyCode::Enter => {
                            let idx = usize::from(win.cursor.y.saturating_sub(4) + win.scroll_y);
                            if idx < win.get_current().content.len() {
                                if let Entry::Folder(dir) = &win.get_current().content[idx] {
                                    win.set_current(dir.clone());
                                }
                            }
                        },
                        _ => {}
                    }
                },
                Event::Resize(width, height) => {
                    win.set_size(width, height);

                    win.scroll_change = true;
                    win.path_change = true;
                    win.cursor.need_update = true;
                },
                _ => {}
            }
        }

        if win.scroll_change {
            win.scroll_change = false;
            print_menu(&win);
            stdout.queue(MoveTo(win.cursor.x, win.cursor.y)).unwrap();
        }

        if win.path_change {
            win.path_change = false;
            print_header(&win);
            stdout.queue(MoveTo(win.cursor.x, win.cursor.y)).unwrap();
        }

        if win.cursor.need_update {
            win.cursor.need_update = false;
            let new_y: u16 = (win.get_current().content.len() + 3).try_into().unwrap_or(win.height.saturating_sub(1));
            if !win.on_dialog && win.cursor.y > new_y && new_y != 3 {
                win.set_cursor(win.cursor.x, new_y);
            }
            stdout.queue(MoveTo(win.cursor.x, win.cursor.y)).unwrap();
        }

        stdout.flush().unwrap();

        thread::sleep(Duration::from_millis(16));
    }
}
