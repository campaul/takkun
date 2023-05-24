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

fn draw_rows(editor: &mut Editor, write: &Box<terminal::Out>) -> io::Result<()> {
    if editor.height == 0 {
        // TODO: this should probably return an error
        return Ok(());
    }

    let window = editor.root.render(editor.width, editor.height);

    for i in 1..editor.height + 1 {
        let line = window.lines.get(i - 1).unwrap();
        write(line.as_bytes())?;

        if line.len() < editor.width {
            write(terminal::CLEAR_LINE)?;
        }

        if i < editor.height {
            write(b"\r\n")?;
        }
    }

    write(position_cursor!(document::Cursor {
        x: window.cursor.x,
        y: window.cursor.y,
    }))?;

    Ok(())
}

fn refresh_screen(editor: &mut Editor, write: &Box<terminal::Out>) -> io::Result<()> {
    write(terminal::HIDE_CURSOR)?;
    write(terminal::ZERO_CURSOR)?;

    draw_rows(editor, write)?;

    write(terminal::SHOW_CURSOR)?;

    Ok(())
}

struct Editor {
    width: usize,
    height: usize,
    root: Box<dyn Component>,
}

impl Editor {
    fn new() -> Editor {
        Editor {
            width: 0,
            height: 0,
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

    fn draw(&mut self, write: &Box<terminal::Out>) -> io::Result<()> {
        refresh_screen(self, write)
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
            if !paused {
                self.draw(&write)?;
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
