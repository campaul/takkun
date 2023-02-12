#![feature(type_alias_impl_trait)]

#[macro_use]
extern crate lazy_static;

mod document;
#[macro_use]
mod terminal;

use std::env;
use std::io;
use std::fs::File;

use document::Document;
use terminal::Event;

fn process_event(mut editor: Editor, event: Event) -> Editor {
    match event {
        Event::Up => {
            editor.document = editor.document.up(editor.width);
        }
        Event::Down => {
            editor.document = editor.document.down(editor.width);
        }
        Event::Left => {
            editor.document = editor.document.left();
        }
        Event::Right => {
            editor.document = editor.document.right();
        }

        Event::PageUp => {
            // TODO: reimplement
        }
        Event::PageDown => {
            // TODO: reimplement
        }
        Event::Home => {
            // TODO: reimplement
        }
        Event::End => {
            // TODO: reimplement
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
    let window = editor.document.window(editor.width, editor.height);
    let placeholder = String::from("~");

    for i in 0..editor.height {
        let line = window.lines.get(i).unwrap_or(&placeholder);
        editor.buffer.extend(line.bytes());
        
        if line.len() < editor.width {
            editor.buffer.extend(terminal::CLEAR_LINE);
        }

        if i < editor.height - 1 {
            editor.buffer.extend(b"\r\n");
        }
    }

    editor.buffer.extend(position_cursor!(window.cursor));

    editor
}

fn refresh_screen(mut editor: Editor) -> Editor {
    editor.buffer.clear();

    editor.buffer.extend(terminal::HIDE_CURSOR);
    editor.buffer.extend(terminal::ZERO_CURSOR);

    editor = draw_rows(editor);

    editor.buffer.extend(terminal::SHOW_CURSOR);

    editor
}

struct Editor {
    width: usize,
    height: usize,
    buffer: Vec<u8>,
    document: Document,
}

impl Editor {
    fn new(file: File) -> Editor {
        Editor {
            width: 0,
            height: 0,
            buffer: vec![],
            document: Document::new(file),
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
    let args: Vec<String> = env::args().collect();

    // TODO: handle no filename or non-existing file
    let file = File::open(&args[1]).unwrap();

    let (read_input, write_output) = terminal::enter_raw_mode()?;

    Editor::new(file).run(read_input, write_output)?;

    terminal::exit_raw_mode()?;

    Ok(())
}
