use std::io;

use crate::document::Cursor;
use crate::document::Document;
use crate::style::styled;
use crate::style::Style;
use crate::terminal::Event;
use crate::ui::Component;
use crate::ui::Window;

pub struct Find {
    child: Box<dyn Component>,
    search: Option<String>,
}

impl Find {
    pub fn new(child: Box<dyn Component>) -> Box<Find> {
        Box::new(Find {
            child: child,
            search: None,
        })
    }
}

impl Component for Find {
    fn update(&mut self, e: Event, width: usize) -> io::Result<bool> {
        if let Event::Find = e {
            self.search = Some(String::new());
            return Ok(true);
        }

        if let Some(_) = self.search {
            match &e {
                Event::Input(c) => {
                    self.search = Some(format!(
                        "{}{}",
                        self.search.clone().unwrap_or("".to_string()),
                        c
                    ))
                }
                Event::Enter => {
                    let search = self.search.clone().unwrap_or(String::new());

                    if search.len() != 0 {
                        self.child.document().find_next(search);
                    }
                }
                Event::Escape => {
                    self.search = None;
                }
                _ => {
                    return Ok(false);
                }
            }

            return Ok(true);
        }

        self.child.update(e, width)
    }

    fn render(&mut self, width: usize, height: usize) -> Window {
        if let None = self.search {
            return self.child.render(width, height);
        }

        let mut child_window = self.child.render(width, height - 1);

        let status = match &self.search {
            Some(s) => format!("FIND: {}", s),
            None => String::new(),
        };

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
