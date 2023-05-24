use std::io;

use crate::document::Cursor;
use crate::document::Document;
use crate::style::styled;
use crate::style::Decoration;
use crate::style::Style;
use crate::terminal::Event;
use crate::ui::Component;
use crate::ui::TextArea;
use crate::ui::Window;

pub struct Tabs {
    children: Vec<Box<dyn Component>>,
    selected: usize,
}

impl Tabs {
    pub fn new(child: Box<dyn Component>) -> Box<dyn Component> {
        Box::new(Tabs {
            children: vec![child],
            selected: 0,
        })
    }
}

impl Tabs {
    fn current_child(&mut self) -> &mut Box<dyn Component> {
        self.children.get_mut(self.selected).unwrap()
    }
}

impl Component for Tabs {
    fn update(&mut self, e: Event, width: usize) -> io::Result<()> {
        match e {
            Event::Next => {
                self.selected = (self.selected + 1) % self.children.len();
            }
            Event::Prev => {
                self.selected = (self.selected + self.children.len() - 1) % self.children.len();
            }
            Event::New => {
                self.children
                    .insert(self.selected + 1, TextArea::new(Document::blank()));
                self.selected += 1;
            }
            Event::Close => {
                self.children.remove(self.selected);
                self.selected = (self.selected + self.children.len() - 1) % self.children.len();
            }
            _ => {
                self.current_child().update(e, width)?;
            }
        }

        Ok(())
    }

    fn render(&mut self, width: usize, height: usize) -> Window {
        let mut child_window = self.current_child().render(width, height - 1);

        let text = format!(
            "{} ({}/{})",
            self.document().name(),
            self.selected + 1,
            self.children.len()
        );
        let left = String::from_utf8(vec![b' '; (width - text.len()) / 2]).unwrap();
        let right = String::from_utf8(vec![b' '; (width - text.len()) / 2]).unwrap();
        let mut pad = String::new();

        if text.len() + left.len() + right.len() < width {
            pad = " ".to_string();
        }

        let header = styled(
            &Style {
                foreground: 7,
                background: 0,
                decoration: vec![Decoration::Italic, Decoration::Underline],
            },
            &format!("{}{}{}{}", left, text, right, pad),
        );

        child_window.lines.insert(0, header);

        Window {
            lines: child_window.lines,
            cursor: Cursor {
                x: child_window.cursor.x,
                y: child_window.cursor.y + 1,
            },
        }
    }

    fn document(&mut self) -> &mut Document {
        self.current_child().document()
    }
}
