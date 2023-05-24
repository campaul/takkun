use std::io;

use crate::document::Cursor;
use crate::document::Document;
use crate::style::styled;
use crate::style::Style;
use crate::terminal::Event;
use crate::ui::Component;
use crate::ui::Window;

#[derive(Clone)]
enum Selection {
    Open(String),
    Save(String),
}

fn set_selection(selection: &Selection, value: String) -> Selection {
    match selection {
        Selection::Open(s) => Selection::Open(value),
        Selection::Save(s) => Selection::Save(value),
    }
}

fn extend_selection(selection: &Selection, value: String) -> Selection {
    match selection {
        Selection::Open(s) => Selection::Open(format!("{}{}", s, value)),
        Selection::Save(s) => Selection::Save(format!("{}{}", s, value)),
    }
}

fn get_selection(selection: &Selection) -> &String {
    match selection {
        Selection::Open(s) => s,
        Selection::Save(s) => s,
    }
}

pub struct FileChooser {
    child: Box<dyn Component>,
    selection: Option<Selection>,
}

impl FileChooser {
    pub fn new(child: Box<dyn Component>) -> Box<FileChooser> {
        Box::new(FileChooser {
            child: child,
            selection: None,
        })
    }
}

impl Component for FileChooser {
    fn update(&mut self, e: Event, width: usize) -> io::Result<()> {
        if let Some(selection) = &self.selection.clone() {
            match &e {
                Event::Input(c) => {
                    self.selection = Some(extend_selection(&selection, c.to_string()));
                }
                Event::Enter => {
                    let filename = get_selection(selection);

                    if filename.len() != 0 {
                        match selection {
                            Selection::Open(_) => {
                                self.child.update(Event::New, width);
                                self.document().open(filename.clone())?;
                                self.selection = None;
                            }
                            Selection::Save(_) => {
                                self.document().set_filename(filename.clone());
                                self.document().save()?;
                                self.selection = None;
                            }
                        }
                    }
                }
                Event::Escape => {
                    self.selection = None;
                }
                _ => {}
            }

            return Ok(());
        } else {
            match &e {
                Event::Open => {
                    self.selection = Some(Selection::Open(String::new()));
                }
                Event::Save => {
                    if let None = self.document().filename {
                        self.selection = Some(Selection::Save(String::new()));
                    }
                }
                _ => {}
            }

            self.child.update(e, width)
        }
    }

    fn render(&mut self, width: usize, height: usize) -> Window {
        if let None = self.selection {
            return self.child.render(width, height);
        }

        let mut child_window = self.child.render(width, height - 1);
        let mut status = String::new();

        if let Some(selection) = &self.selection {
            status = match selection {
                Selection::Open(s) => format!("OPEN: {}", s),
                Selection::Save(s) => format!("SAVE AS: {}", s),
            };
        }

        let footer = styled(
            &Style {
                foreground: 7,
                background: 12,
                decoration: vec![],
            },
            &format!(" {} ", status),
        );

        child_window.lines.push(footer);

        Window {
            lines: child_window.lines,
            cursor: Cursor {
                x: child_window.cursor.x,
                y: child_window.cursor.y,
            },
        }
    }

    fn document(&mut self) -> &mut Document {
        self.child.document()
    }
}
