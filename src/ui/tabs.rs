use std::io;

use crate::document::Cursor;
use crate::document::Document;
use crate::style::styled;
use crate::style::Decoration;
use crate::style::Style;
use crate::terminal::Event;
use crate::ui::Component;
use crate::ui::Window;

pub struct Tabs {
    children: Vec<Box<dyn Component>>,
}

impl Tabs {
    pub fn new(child: Box<dyn Component>) -> Box<Tabs> {
        Box::new(Tabs {
            children: vec![child],
        })
    }
}

impl Component for Tabs {
    fn update(&mut self, e: Event, width: usize) -> io::Result<()> {
        self.children.get_mut(0).unwrap().update(e, width)
    }

    fn render(&mut self, width: usize, height: usize) -> Window {
        let mut child_window = self.children.get_mut(0).unwrap().render(width, height - 1);

        let text = self.document().name();
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
        self.children.get_mut(0).unwrap().document()
    }
}
