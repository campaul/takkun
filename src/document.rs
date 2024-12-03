use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use crate::style::styled;
use crate::style::Style;

#[derive(Copy, Clone)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone)]
pub struct Cell {
    grapheme: String,
    width: usize,
    style: Style,
}

#[derive(Clone)]
pub struct Row {
    cells: Vec<Cell>,
}

impl Row {
    fn new() -> Row {
        Row { cells: vec![] }
    }

    fn insert_str(&mut self, position: usize, s: &str) {
        self.cells.splice(position..position, cells(s).cells);
    }

    fn split_at(&self, position: usize) -> (Row, Row) {
        let (left, right) = self.cells.split_at(position);
        (
            Row {
                cells: left.to_vec(),
            },
            Row {
                cells: right.to_vec(),
            },
        )
    }

    fn append(&mut self, row: Row) {
        self.cells.extend(row.cells);
    }

    fn remove(&mut self, position: usize) {
        self.cells.remove(position);
    }

    fn as_string(&self) -> String {
        let mut line = String::new();

        for cell in self.cells.iter() {
            line = line + &cell.grapheme;
        }

        line
    }

    fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn split(&self, max_width: usize, end: &str) -> Vec<String> {
        let mut display_lines: Vec<String> = vec![];
        let mut line = String::new();
        let mut width = 0;
        let mut style = &Style {
            foreground: 7,
            background: 234,
            decoration: vec![],
        };

        line.push_str(&styled(style, &String::new()));

        for cell in self.cells.iter() {
            if &cell.style != style {
                style = &cell.style;
                line.push_str(&styled(&style, &cell.grapheme));
            } else {
                line.push_str(&cell.grapheme);
            }

            if width + cell.width < max_width {
                width += cell.width;
            } else {
                display_lines.push(line);
                line = String::new();
                width = 0;
            }
        }

        if width < max_width {
            line = line + end;
        }

        display_lines.push(line);

        display_lines
    }

    pub fn match_indices(&self, pattern: &str) -> Vec<usize> {
        if pattern.len() > self.cells.len() {
            return vec![];
        }

        let mut matches = vec![];
        let pattern_graphemes: Vec<&str> = pattern.graphemes(false).collect();

        for i in 0..self.cells.len() - pattern_graphemes.len() {
            let mut does_match = true;
            for j in 0..pattern_graphemes.len() {
                if self.cells[i + j].grapheme != pattern_graphemes[j] {
                    does_match = false;
                    break;
                }
            }
            if does_match {
                matches.push(i);
            }
        }

        matches
    }
}

pub fn cells(line: &str) -> Row {
    Row {
        cells: line
            .graphemes(false)
            .map(|g| {
                let grapheme = g.to_string();

                if g == String::from("\t") {
                    Cell {
                        grapheme: grapheme,
                        width: 4,
                        style: Style {
                            foreground: 7,
                            background: 234,
                            decoration: vec![],
                        },
                    }
                } else {
                    Cell {
                        grapheme: grapheme,
                        width: g.width(),
                        style: Style {
                            foreground: 7,
                            background: 234,
                            decoration: vec![],
                        },
                    }
                }
            })
            .collect(),
    }
}

pub struct Document {
    pub rows: Vec<Row>,
    pub cursor: Cursor,
    pub filename: Option<String>,
}

impl Document {
    pub fn blank() -> Document {
        Document {
            rows: vec![],
            cursor: Cursor { x: 0, y: 0 },
            filename: None,
        }
    }

    pub fn open(&mut self, filename: String) -> io::Result<()> {
        let mut contents = String::new();

        if Path::new(&filename).exists() {
            let mut file = File::open(filename.clone())?;
            file.read_to_string(&mut contents)?;
        }

        self.rows = contents.lines().map(cells).collect();
        self.cursor = Cursor { x: 0, y: 0 };
        self.filename = Some(filename);

        Ok(())
    }

    pub fn name(&self) -> String {
        self.filename.clone().unwrap_or("New File".to_string())
    }

    pub fn insert(&mut self, c: &String) {
        assert!(c.len() == 1);

        if self.rows.len() == 0 {
            self.rows.push(Row::new());
        }

        self.rows[self.cursor.y].insert_str(self.cursor.x, &c);
        self.cursor.x += 1;
    }

    pub fn insert_line(&mut self) {
        if self.rows.len() == 0 {
            return;
        }

        let row = self.rows.remove(self.cursor.y);
        let (first, last) = row.split_at(self.cursor.x);

        self.rows.insert(self.cursor.y, last);
        self.rows.insert(self.cursor.y, first);

        self.cursor.y += 1;
        self.cursor.x = 0;
    }

    pub fn delete_next(&mut self) {
        if self.rows.len() == 0 {
            return;
        }

        if !(self.on_last_line() && self.on_last_char()) {
            self.right();
            self.delete_prev();
        }
    }

    pub fn delete_prev(&mut self) {
        if self.on_first_char() && !self.on_first_line() {
            let prev = self.rows.remove(self.cursor.y);

            self.cursor.y -= 1;
            self.cursor.x = self.current_line_len();

            self.rows[self.cursor.y].append(prev);
        } else if !self.on_first_char() {
            self.cursor.x -= 1;
            self.rows[self.cursor.y].remove(self.cursor.x);
        }
    }

    pub fn tab(&mut self) {
        for _ in 0..4 {
            self.insert(&" ".to_string());
        }
    }

    pub fn set_filename(&mut self, filename: String) {
        self.filename = Some(filename);
    }

    pub fn save(&self) -> std::io::Result<()> {
        if let Some(filename) = &self.filename {
            let mut buffer = File::create(filename)?;

            for row in &self.rows {
                buffer.write_all(row.as_string().as_bytes())?;
                buffer.write_all(&[b'\n'])?;
            }
        }

        Ok(())
    }

    pub fn on_first_line(&self) -> bool {
        self.cursor.y == 0
    }

    pub fn on_last_line(&self) -> bool {
        self.cursor.y == self.rows.len() - 1
    }

    fn on_first_char(&self) -> bool {
        self.cursor.x == 0
    }

    fn on_last_char(&self) -> bool {
        self.cursor.x == self.current_line_len()
    }

    pub fn current_line_len(&self) -> usize {
        if self.rows.len() == 0 {
            return 0;
        }

        self.rows[self.cursor.y].len()
    }

    pub fn up(&mut self) {
        if self.on_first_line() {
            self.start_of_line();
        } else {
            self.cursor.y -= 1;
            self.cursor.x = std::cmp::min(self.cursor.x, self.current_line_len());
        }
    }

    pub fn down(&mut self) {
        if self.on_last_line() {
            self.end_of_line();
        } else {
            self.cursor.y += 1;
            self.cursor.x = std::cmp::min(self.cursor.x, self.current_line_len());
        }
    }

    pub fn left(&mut self) {
        if self.on_first_char() && !self.on_first_line() {
            self.cursor.y -= 1;
            self.cursor.x = self.rows[self.cursor.y].len();
        } else if !self.on_first_char() {
            self.cursor.x -= 1;
        }
    }

    pub fn right(&mut self) {
        if self.rows.len() == 0 {
            return;
        }

        if self.on_last_char() && !self.on_last_line() {
            self.cursor.y += 1;
            self.cursor.x = 0;
        } else if !self.on_last_char() {
            self.cursor.x += 1;
        }
    }

    pub fn end_of_line(&mut self) {
        self.cursor.x = self.current_line_len();
    }

    pub fn start_of_line(&mut self) {
        self.cursor.x = 0;
    }

    pub fn find_next(&mut self, text: String) {
        let mut matches: Vec<(usize, usize)> = vec![];

        for i in 0..self.rows.len() {
            for m in self.rows[i].match_indices(&text) {
                matches.push((m, i));
            }
        }

        if matches.len() > 0 {
            let mut next = matches[0];

            for m in matches {
                if m.1 == self.cursor.y && m.0 > self.cursor.x {
                    next = m;
                    break;
                } else if m.1 > self.cursor.y {
                    next = m;
                    break;
                }
            }

            self.cursor.x = next.0;
            self.cursor.y = next.1;
        }
    }

    pub fn cursor_display_x(&self) -> usize {
        let mut display_len = 0;

        for i in 0..self.cursor.x {
            display_len += self.rows[self.cursor.y].cells[i].width;
        }

        display_len
    }
}

#[cfg(test)]
mod tests {
    use crate::document::Document;

    #[test]
    fn current_line_len() {
        let mut document = Document::blank();

        // Returns 0 when there is no current line
        assert_eq!(document.current_line_len(), 0);

        // Returns 1 when the line has 1 character
        document.insert(&String::from(" "));
        assert_eq!(document.current_line_len(), 1);

        // Returns 0 when current line is empty
        document.delete_prev();
        assert_eq!(document.current_line_len(), 0);
    }
}
