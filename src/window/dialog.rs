use Event_Handler::event_handler::{
    click::ClickEvent,
    zone::rectangule::Rect,
    action::ActionHandle
};
use crate::window::window::Window;
use std::io::{StdoutLock, Write};
use crossterm::{
    QueueableCommand,
    cursor::MoveTo
};

#[derive(Clone)]
pub struct Dialog {
    pub text: String,
    pub x: u16,
    pub y: u16,
    pub h: u16,
    pub w: u16
}

impl Dialog {
    pub fn new(win: &Window, text: String) -> Self {
        let text_len = text.len() as u16;
        let x = win.width / 2 - text_len;
        let y = win.height / 2 - 1;

        Self {
            text,
            x,
            y,
            h: x + text_len,
            w: y + 2
        }
    }

    pub fn event<T: ActionHandle + 'static>(&mut self, _do: T) -> ClickEvent {
        ClickEvent::new(
            Rect::new(self.x, self.y, self.h + 2, self.w),
            true,
            _do
        )
    }

    pub fn draw(&self, out: *mut StdoutLock) {
        let stdout = unsafe { &mut (*out) };
        let fill_all_block = "─".repeat(usize::from(self.h - self.x));

        stdout.queue(MoveTo(self.x, self.y)).unwrap();
        stdout.write("┌".as_bytes()).unwrap();
        stdout.write(fill_all_block.as_bytes()).unwrap();
        stdout.write("┐".as_bytes()).unwrap();

        stdout.queue(MoveTo(self.x, self.y + 1)).unwrap();
        stdout.write("│".as_bytes()).unwrap();
        stdout.write(self.text.as_bytes()).unwrap();
        stdout.write("│".as_bytes()).unwrap();

        stdout.queue(MoveTo(self.x, self.w)).unwrap();
        stdout.write("└".as_bytes()).unwrap();
        stdout.write(fill_all_block.as_bytes()).unwrap();
        stdout.write("┘".as_bytes()).unwrap();
    }
}
