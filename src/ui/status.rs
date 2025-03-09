use std::io;

use crate::document::Cursor;
use crate::document::Document;
use crate::style::styled;
use crate::style::Style;
use crate::terminal::Event;
use crate::ui::Component;
use crate::ui::Window;

pub struct Status {
    child: Box<dyn Component>,
    error: Option<String>,
}

impl Status {
    pub fn new(child: Box<dyn Component>) -> Box<Status> {
        Box::new(Status {
            child: child,
            error: None,
        })
    }
}

impl Component for Status {
    fn update(&mut self, e: &Event, width: usize) -> io::Result<bool> {
        if let Event::Error(error) = e {
            self.error = Some(error.to_string());
        }

        if let Some(_) = &self.error {
            match &e {
                Event::Escape => {
                    self.error = None;
                }
                _ => {}
            }
        } else {
            let u = self.child.update(e, width);
            if let Err(error) = u {
                self.error = Some(error.to_string());
            } else {
                return u;
            }
        }

        Ok(false)
    }

    fn render(&mut self, width: usize, height: usize) -> Window {
        let mut child_window = self.child.render(width, height - 1);

        let mut status = String::new();

        if let Some(e) = &self.error {
            status = format!("ERROR: {}", e);
        }

        let position = format!(
            "{}:{}",
            self.child.document().cursor.y + 1,
            self.child.document().cursor.x + 1
        );
        let padding =
            String::from_utf8(vec![b' '; width - status.len() - position.len() - 2]).unwrap();

        let footer_style = match self.error {
            Some(_) => Style {
                foreground: 7,
                background: 9,
                decoration: vec![],
            },
            None => Style {
                foreground: 0,
                background: 7,
                decoration: vec![],
            },
        };

        let footer = styled(
            &footer_style,
            &format!(" {}{}{} ", status, padding, position),
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
