use crossterm::{
    self, terminal::{self, Clear, ClearType}, cursor::MoveTo,
    QueueableCommand,
    event::{self, Event, KeyCode}
};

use std::{
    time::Duration, io::{stdout, Write}, thread,
    process::exit, env::args
};

use content_7z::{
    files::entry::Entry,
    window::{
        window::Window,
        dialog::Dialog
    }
};

use content_7z::zip_manager::manager::ZipManager;

use Event_Handler::event_handler::{
    event_handler::EventsHandler,
    event::HowEvent,
    action::ActionHandle,
};

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

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        eprintln!("Usage:\n\t{} {{7zip file}}", &args[0]);
        exit(-1);
    }

    let mut stdout = stdout().lock();

    let mut win = Window::new(&mut stdout);
    let manager = ZipManager::process(&args[1]);
    if manager.err != "" {
        eprintln!("Process Error: {}", manager.err);
        exit(-1);
    }

    win.assing_manager(manager);

    print_header(&mut win);

    let mut handler = EventsHandler::new();

    struct controller;

    impl HowEvent for controller {
        fn call(&mut self, ev: &Event) -> bool {
            false
        }
    }

    struct ShowMsg<'a> {
        win: *mut Window<'a>,
        dialog: Dialog
    }

    impl<'a> ShowMsg<'a> {
        fn new(win: *mut Window<'a>) -> Self {
            unsafe {
                Self {
                    win,
                    dialog: Dialog::new(&mut(*win), String::new())
                }
            }
        }

        fn add_dialog(&mut self, dialog: Dialog) {
            self.dialog = dialog;
        }
    }

    impl<'a> ActionHandle for ShowMsg<'a> {
        fn accept(&mut self) -> bool {
            let win = unsafe {
                &mut (*self.win)
            };

            print_menu(&win);
            print_header(&win);

            win.on_dialog = false;

            true
        }

        fn denied(&mut self) -> bool {
            let win = unsafe { &mut (*self.win) };
            self.dialog.draw(win.writer);
            win.on_dialog = true;
            false
        }
    }

    'mainLoop:
    loop {
        while event::poll(Duration::ZERO).unwrap() {
            let ev = event::read().unwrap();
            handler.update(&ev);
            if win.on_dialog {
                continue;
            }
            match ev {
                Event::Key(ev) => {
                    match ev.code {
                        KeyCode::Esc => break 'mainLoop,
                        KeyCode::Up => win.move_up(),
                        KeyCode::Down => win.move_down(),
                        KeyCode::Right => win.move_right(),
                        KeyCode::Left => win.move_left(),

                        KeyCode::Char('s') => {
                            let mut action = ShowMsg::new(&mut win);
                            let mut dialog = Dialog::new(&mut win, String::from("Hola, como estas?"));
                            action.add_dialog(dialog.clone());
                            handler.add_event(dialog.event(action));
                            dialog.draw(&mut stdout);
                        },

                        KeyCode::Backspace => win.back_current(),
                        KeyCode::Enter => {
                            if usize::from(win.cursor.y - 4 + win.scroll_y) < win.get_current().content.len() {
                                match &win.get_current().content[usize::from(win.cursor.y - 4 + win.scroll_y)] {
                                    Entry::Folder(dir) => win.set_current(dir.clone()),
                                    Entry::File(file_name) => {
                                        let path = win.plain_current() + "/" + file_name;
                                        let mut action = ShowMsg::new(&mut win);
                                        let mut dialog = Dialog::new(&mut win, String::from(path));
                                        action.add_dialog(dialog.clone());
                                        handler.add_event(dialog.event(action));
                                        dialog.draw(&mut stdout);
                                    },
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

        if win.cursor.need_update {
            win.cursor.need_update = false;
            handler.update_cursor(win.cursor.x, win.cursor.y);
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
