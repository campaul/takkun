use std::io;

use crate::document::Cursor;
use crate::document::Document;
use crate::style::styled;
use crate::style::Style;
use crate::terminal::Event;
use crate::terminal::CLEAR_LINE;
use crate::ui::Component;
use crate::ui::Window;

pub struct TextArea {
    document: Document,
    window_offset: usize,
}

impl TextArea {
    pub fn new(document: Document) -> Box<TextArea> {
        Box::new(TextArea {
            document: document,
            window_offset: 0,
        })
    }

    pub fn up(&mut self, width: usize) {
        if self.document.cursor.x >= width {
            self.document.cursor.x -= width;
        } else {
            self.document.up();
            self.document.cursor.x += (self.document.current_line_len() / width) * width;
        }
    }

    pub fn down(&mut self, width: usize) {
        if self.document.rows.len() == 0 {
            return;
        }

        if self.document.cursor.x + width < self.document.current_line_len() {
            self.document.cursor.x += width;
        } else {
            self.document.cursor.x = self.document.cursor.x % width;
            self.document.down();
        }
    }
}

impl Component for TextArea {
    fn update(&mut self, event: &Event, width: usize) -> io::Result<bool> {
        match event {
            Event::Input(c) => {
                self.document.insert(c);
            }

            Event::Up => {
                self.up(width);
            }
            Event::Down => {
                self.down(width);
            }
            Event::Left => {
                self.document.left();
            }
            Event::Right => {
                self.document.right();
            }

            Event::PageUp => {
                // TODO: reimplement
            }
            Event::PageDown => {
                // TODO: reimplement
            }
            Event::Home => {
                self.document.start_of_line();
            }
            Event::End => {
                self.document.end_of_line();
            }

            Event::Tab => {
                self.document.tab();
            }
            Event::Delete => {
                self.document.delete_next();
            }
            Event::Backspace => {
                self.document.delete_prev();
            }
            Event::Enter => {
                self.document.insert_line();
            }

            _ => {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn render(&mut self, width: usize, height: usize) -> Window {
        let mut lines: Vec<String> = vec![];
        let mut cursor = Cursor { x: 0, y: 0 };

        if width == 0 {
            return Window {
                lines: lines,
                cursor: cursor,
            };
        }

        for (i, row) in self.document.rows.iter().enumerate() {
            let split_lines = row.split(width, std::str::from_utf8(CLEAR_LINE).unwrap());

            if i == self.document.cursor.y {
                cursor.x = self.document.cursor_display_x() % width;
                cursor.y = lines.len() + self.document.cursor_display_x() / width;
            }

            lines.extend(split_lines);
        }

        if cursor.y < self.window_offset {
            self.window_offset = cursor.y;
        }

        if cursor.y > self.window_offset + height - 1 {
            self.window_offset = cursor.y - height + 1;
        }

        cursor.y = cursor.y - self.window_offset;

        let last_line = std::cmp::min(self.window_offset + height, lines.len());

        let visible_lines = &mut lines[self.window_offset..last_line].to_vec();

        for _ in last_line..(self.window_offset + height) {
            visible_lines.push(format!("~{}", std::str::from_utf8(CLEAR_LINE).unwrap()));
        }

        let styled_lines: Vec<String> = visible_lines
            .into_iter()
            .map(|l| {
                styled(
                    &Style {
                        foreground: 7,
                        background: 234,
                        decoration: vec![],
                    },
                    &l,
                )
            })
            .collect();

        Window {
            lines: styled_lines,
            cursor: cursor,
        }
    }

    fn document(&mut self) -> &mut Document {
        &mut self.document
    }
}
