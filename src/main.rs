#![feature(type_alias_impl_trait)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod terminal;

use std::io;

use terminal::Event;

fn process_event(mut editor: Editor, event: Event) -> Editor {
    match event {
        Event::Up => {
            if editor.cursor.y > 0 {
                editor.cursor.y -= 1;
            }
        }
        Event::Down => {
            if editor.cursor.y < editor.height - 1 {
                editor.cursor.y += 1;
            }
        }
        Event::Left => {
            if editor.cursor.x > 0 {
                editor.cursor.x -= 1;
            }
        }
        Event::Right => {
            if editor.cursor.x < editor.width - 1 {
                editor.cursor.x += 1;
            }
        }

        Event::PageUp => {
            editor.cursor.y = 0;
        }
        Event::PageDown => {
            editor.cursor.y = editor.height - 1;
        }
        Event::Home => {
            editor.cursor.x = 0;
        }
        Event::End => {
            editor.cursor.x = editor.width - 1;
        }

        Event::Delete => {}
        Event::Escape => {}

        Event::Nothing => {}

        Event::Resize(width, height) => {
            editor.width = width;
            editor.height = height;
        }

        Event::Exit => {}
        Event::Error(_) => {}
    }

    editor
}

fn draw_rows(mut editor: Editor) -> Editor {
    for y in 0..editor.height {
        if y == editor.height / 3 {
            editor.buffer.extend(b"Hello there");
        } else {
            editor.buffer.extend(b"~");
        }

        // TODO: what if lines are longer than the screen width?
        editor.buffer.extend(terminal::CLEAR_LINE);

        if y < editor.height - 1 {
            editor.buffer.extend(b"\r\n");
        }
    }

    editor
}

fn refresh_screen(mut editor: Editor) -> Editor {
    editor.buffer.clear();

    editor.buffer.extend(terminal::HIDE_CURSOR);
    editor.buffer.extend(terminal::ZERO_CURSOR);

    editor = draw_rows(editor);

    editor.buffer.extend(position_cursor!(editor.cursor));

    editor.buffer.extend(terminal::SHOW_CURSOR);

    editor
}

struct Cursor {
    x: u16,
    y: u16,
}

struct Editor {
    width: u16,
    height: u16,
    cursor: Cursor,
    buffer: Vec<u8>,
}

impl Editor {
    fn new() -> Editor {
        Editor {
            width: 0,
            height: 0,
            cursor: Cursor { x: 0, y: 0 },
            buffer: vec![],
        }
    }

    fn update(self, event: Event) -> Editor {
        process_event(self, event)
    }

    fn draw(self) -> Editor {
        refresh_screen(self)
    }

    fn run(self, read: terminal::In, write: terminal::Out) -> io::Result<()> {
        let mut editor = self;

        loop {
            editor = editor.draw();

            write(&editor.buffer)?;

            let event = read();

            match event {
                Event::Exit => break,
                _ => editor = editor.update(event),
            };
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let (read_input, write_output) = terminal::enter_raw_mode()?;

    Editor::new().run(read_input, write_output)?;

    terminal::exit_raw_mode()?;

    Ok(())
}
