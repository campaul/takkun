#![feature(type_alias_impl_trait)]

mod document;
mod style;
#[macro_use]
mod terminal;

use std::env;
use std::io;

use document::Document;
use style::styled;
use style::Style;
use terminal::Event;

fn process_search(mut editor: Editor, event: Event) -> Option<Editor> {
    match event {
        Event::Input(c) => {
            editor.search = Some(format!("{}{}", editor.search.unwrap_or("".to_string()), c))
        }
        Event::Enter => {
            let search = editor.search.clone().unwrap_or(String::new());

            if search.len() != 0 {
                editor.document = editor.document.find_next(search);
            }
        }
        Event::Escape => {
            editor.search = None;
        }
        _ => {}
    }

    Some(editor)
}

fn process_event(mut editor: Editor, event: Event) -> Option<Editor> {
    if let Some(_) = &editor.search {
        return process_search(editor, event);
    }

    match event {
        Event::Input(c) => {
            editor.document = editor.document.insert(c);
        }
        Event::Control(c) => match c.as_str() {
            "o" => {
                if let Err(e) = editor.document.save() {
                    editor.error = Some(e);
                }
            }
            "q" => return None,
            "f" => editor.search = Some(String::new()),
            _ => {}
        },

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
            editor.document = editor.document.start_of_line();
        }
        Event::End => editor.document = editor.document.end_of_line(),

        Event::Tab => {
            editor.document = editor.document.tab();
        }
        Event::Delete => {
            editor.document = editor.document.delete_next();
        }
        Event::Backspace => {
            editor.document = editor.document.delete_prev();
        }
        Event::Escape => {}
        Event::Enter => {
            editor.document = editor.document.insert_line();
        }

        Event::Pause => {
            editor.paused = true;
            terminal::pause().unwrap();
        }
        Event::Resume => {
            editor.paused = false;
            terminal::resume().unwrap();
        }

        Event::Resize => {
            let (width, height) = terminal::get_window_size().unwrap();
            editor.width = width;
            editor.height = height;
        }

        Event::Error(_) => {}

        _ => {}
    }

    Some(editor)
}

fn header(text: String, width: usize, style: Style) -> String {
    let left = String::from_utf8(vec![b' '; (width - text.len()) / 2]).unwrap();
    let right = String::from_utf8(vec![b' '; (width - text.len()) / 2]).unwrap();
    let mut pad = String::new();

    if text.len() + left.len() + right.len() < width {
        pad = " ".to_string();
    }

    styled(style, format!("{}{}{}{}\r\n", left, text, right, pad))
}

fn footer(status: String, row: usize, col: usize, width: usize, style: Style) -> String {
    let position = format!("{}:{}", col + 1, row + 1);
    let padding = String::from_utf8(vec![b' '; width - status.len() - position.len() - 2]).unwrap();

    styled(style, format!(" {}{}{} ", status, padding, position))
}

fn draw_rows(mut editor: Editor) -> Editor {
    if editor.height == 0 {
        return editor;
    }

    let header_style = Style {
        foreground: 0,
        background: 7,
    };

    let footer_style = match editor.search {
        Some(_) => Style {
            foreground: 7,
            background: 12,
        },
        None => Style {
            foreground: 0,
            background: 7,
        },
    };

    let window = editor.document.window(editor.width, editor.height - 2);
    let placeholder = String::from("~");

    let name = editor
        .document
        .filename
        .clone()
        .unwrap_or("New File".to_string());
    editor
        .buffer
        .extend(header(name, editor.width, header_style).bytes());

    for i in 1..editor.height - 1 {
        let line = window.lines.get(i - 1).unwrap_or(&placeholder);
        editor.buffer.extend(line.bytes());

        if line.len() < editor.width {
            editor.buffer.extend(terminal::CLEAR_LINE);
        }

        if i < editor.height - 1 {
            editor.buffer.extend(b"\r\n");
        }
    }

    let status = match &editor.search {
        Some(s) => format!("FIND: {}", s),
        None => String::new(),
    };
    editor.buffer.extend(
        footer(
            status,
            editor.document.cursor.x,
            editor.document.cursor.y,
            editor.width,
            footer_style,
        )
        .bytes(),
    );

    editor.buffer.extend(position_cursor!(document::Cursor {
        x: window.cursor.x,
        y: window.cursor.y + 1,
    }));

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
    error: Option<io::Error>,
    search: Option<String>,
    paused: bool,
}

impl Editor {
    fn new(filename: Option<String>) -> Editor {
        match Document::new(filename) {
            Ok(document) => Editor {
                width: 0,
                height: 0,
                buffer: vec![],
                document: document,
                error: None,
                search: None,
                paused: false,
            },
            Err(e) => Editor {
                width: 0,
                height: 0,
                buffer: vec![],
                document: Document::blank(),
                error: Some(e),
                search: None,
                paused: false,
            },
        }
    }

    fn update(self, event: Event) -> Option<Editor> {
        process_event(self, event)
    }

    fn draw(self) -> Editor {
        refresh_screen(self)
    }

    fn run(self, read: terminal::In, write: terminal::Out) -> io::Result<()> {
        let mut editor = self;

        loop {
            editor = editor.draw();

            if !editor.paused {
                write(&editor.buffer)?;
            }

            match editor.update(read()) {
                Some(e) => editor = e,
                None => break,
            }
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let (read_input, write_output) = terminal::init()?;

    Editor::new(args.get(1).cloned()).run(read_input, write_output)?;

    terminal::exit()?;

    Ok(())
}
