mod document;
mod style;
#[macro_use]
mod terminal;
mod ui;

use std::env;
use std::io;

use document::Document;
use terminal::Event;
use ui::Component;
use ui::FileChooser;
use ui::Find;
use ui::Status;
use ui::Tabs;
use ui::TextArea;

fn draw_rows(editor: &mut Editor) {
    if editor.height == 0 {
        return;
    }

    let window = editor.root.render(editor.width, editor.height);
    let mut line_buffer: Vec<u8> = vec![];

    for i in 1..editor.height + 1 {
        let line = window.lines.get(i - 1).unwrap();
        line_buffer.extend(line.bytes());

        if line.len() < editor.width {
            line_buffer.extend(terminal::CLEAR_LINE);
        }

        if i < editor.height {
            line_buffer.extend(b"\r\n");
        }
    }

    editor.buffer.extend(line_buffer);

    editor.buffer.extend(position_cursor!(document::Cursor {
        x: window.cursor.x,
        y: window.cursor.y,
    }));
}

fn refresh_screen(editor: &mut Editor) {
    editor.buffer.clear();

    editor.buffer.extend(terminal::HIDE_CURSOR);
    editor.buffer.extend(terminal::ZERO_CURSOR);

    draw_rows(editor);

    editor.buffer.extend(terminal::SHOW_CURSOR);
}

struct Editor {
    width: usize,
    height: usize,
    buffer: Vec<u8>,
    root: Box<dyn Component>,
}

impl Editor {
    fn new() -> Editor {
        Editor {
            width: 0,
            height: 0,
            buffer: vec![],
            root: Editor::create_root(Document::blank()),
        }
    }

    fn create_root(document: Document) -> Box<dyn Component> {
        Status::new(FileChooser::new(Find::new(Tabs::new(TextArea::new(
            document,
        )))))
    }

    fn update(&mut self, event: Event) -> io::Result<()> {
        match event {
            Event::Resize(width, height) => {
                self.width = width;
                self.height = height;
            }

            _ => self.root.update(event, self.width)?,
        }

        Ok(())
    }

    fn draw(&mut self) {
        refresh_screen(self);
    }

    fn run(
        mut self,
        filename: Option<String>,
        read: Box<terminal::In>,
        write: Box<terminal::Out>,
    ) -> io::Result<()> {
        let mut paused = false;

        if let Some(f) = filename {
            if let Err(e) = self.root.document().open(f) {
                self.update(Event::Error(e.to_string()))?;
            }
        }

        loop {
            self.draw();

            if !paused {
                write(&self.buffer)?;
            }

            match read() {
                Event::Pause => {
                    paused = true;
                    terminal::pause()?;
                }
                Event::Resume => {
                    paused = false;
                    terminal::resume()?;
                }
                Event::Exit => {
                    // TODO: propagate this event to check for unsaved files
                    break;
                }
                e => self.update(e)?,
            }
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).cloned();
    let (read_input, write_output) = terminal::init()?;

    Editor::new().run(filename, read_input, write_output)?;

    terminal::exit()?;

    Ok(())
}
