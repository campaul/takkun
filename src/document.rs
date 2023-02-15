use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

// Split a line into multiple lines based on a maximum width
// TODO: support UTF-8 instead of just ASCII
fn split(line: &String, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = vec![];

    if width == 0 {
        return lines;
    }

    for i in 0..line.len() / width {
        lines.push(line[i * width..i * width + width].into());
    }

    lines.push(line[line.len() - line.len() % width..line.len()].into());

    lines
}

pub struct Cursor {
    pub x: usize,
    pub y: usize,
}

pub struct Document {
    rows: Vec<String>,
    window_offset: usize,
    cursor: Cursor,
    filename: Option<String>,
}

pub struct Window {
    pub lines: Vec<String>,
    pub cursor: Cursor,
}

impl Document {
    pub fn new(filename: Option<String>) -> io::Result<Document> {
        match filename {
            Some(name) => Document::open(name),
            None => Ok(Document::blank()),
        }
    }

    pub fn blank() -> Document {
        Document {
            rows: vec![],
            window_offset: 0,
            cursor: Cursor { x: 0, y: 0 },
            filename: None,
        }
    }

    pub fn open(filename: String) -> io::Result<Document> {
        let mut contents = String::new();

        if Path::new(&filename).exists() {
            let mut file = File::open(filename.clone())?;
            file.read_to_string(&mut contents).unwrap();
        }

        Ok(Document {
            rows: contents.lines().map(String::from).collect(),
            window_offset: 0,
            cursor: Cursor { x: 0, y: 0 },
            filename: Some(filename),
        })
    }

    pub fn insert(mut self, c: String) -> Document {
        assert!(c.len() == 1);

        if self.rows.len() == 0 {
            self.rows.push(String::new());
        }

        self.rows[self.cursor.y].insert_str(self.cursor.x, &c);
        self.cursor.x += 1;

        self
    }

    pub fn insert_line(mut self) -> Document {
        if self.rows.len() == 0 {
            return self;
        }

        let row = self.rows.remove(self.cursor.y);
        let (first, last) = row.split_at(self.cursor.x);

        self.rows.insert(self.cursor.y, last.to_string());
        self.rows.insert(self.cursor.y, first.to_string());

        self.cursor.y += 1;
        self.cursor.x = 0;

        self
    }

    pub fn delete_next(self) -> Document {
        if self.rows.len() == 0 {
            return self;
        }

        if !(self.on_last_line() && self.on_last_char()) {
            return self.right().delete_prev();
        }

        self
    }

    pub fn delete_prev(mut self) -> Document {
        if self.on_first_char() && !self.on_first_line() {
            let prev = self.rows.remove(self.cursor.y);

            self.cursor.y -= 1;
            self.cursor.x = self.current_line_len();

            self.rows[self.cursor.y] = format!("{}{}", self.rows[self.cursor.y], prev);
        } else if !self.on_first_char() {
            self.cursor.x -= 1;
            self.rows[self.cursor.y].remove(self.cursor.x);
        }

        self
    }

    pub fn tab(self) -> Document {
        let mut document = self;

        for _ in 0..4 {
            document = document.insert(" ".to_string());
        }

        document
    }

    pub fn save(&self) -> std::io::Result<()> {
        let mut out = String::new();

        for row in &self.rows {
            out = format!("{}{}\n", out, row);
        }

        if let Some(filename) = &self.filename {
            fs::write(filename, out)?;
        }

        Ok(())
    }

    pub fn window(&mut self, width: usize, height: usize) -> Window {
        let mut lines: Vec<String> = vec![];
        let mut cursor = Cursor { x: 0, y: 0 };

        if width == 0 {
            return Window {
                lines: lines,
                cursor: cursor,
            };
        }

        for i in 0..self.rows.len() {
            let line = &self.rows[i];
            let split_lines = split(line, width);

            if i == self.cursor.y {
                cursor.x = self.cursor.x % width;
                cursor.y = lines.len() + self.cursor.x / width;
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

        Window {
            lines: lines[self.window_offset..last_line].to_vec(),
            cursor: cursor,
        }
    }

    fn on_first_line(&self) -> bool {
        self.cursor.y == 0
    }

    fn on_last_line(&self) -> bool {
        self.cursor.y == self.rows.len() - 1
    }

    fn on_first_char(&self) -> bool {
        self.cursor.x == 0
    }

    fn on_last_char(&self) -> bool {
        self.cursor.x == self.current_line_len()
    }

    fn current_line_len(&self) -> usize {
        self.rows[self.cursor.y].len()
    }

    pub fn left(mut self) -> Document {
        if self.on_first_char() && !self.on_first_line() {
            self.cursor.y -= 1;
            self.cursor.x = self.rows[self.cursor.y].len();
        } else if !self.on_first_char() {
            self.cursor.x -= 1;
        }

        self
    }

    pub fn right(mut self) -> Document {
        if self.rows.len() == 0 {
            return self;
        }

        if self.on_last_char() && !self.on_last_line() {
            self.cursor.y += 1;
            self.cursor.x = 0;
        } else if !self.on_last_char() {
            self.cursor.x += 1;
        }

        self
    }

    pub fn up(mut self, width: usize) -> Document {
        if self.cursor.x >= width {
            self.cursor.x -= width;
        } else if self.on_first_line() {
            self.cursor.x = 0;
        } else {
            self.cursor.y -= 1;
            self.cursor.x = std::cmp::min((self.cursor.x / width) * width, self.current_line_len());
        }

        self
    }

    pub fn down(mut self, width: usize) -> Document {
        if self.rows.len() == 0 {
            return self;
        }

        if self.cursor.x + width < self.current_line_len() {
            self.cursor.x += width;
        } else if self.on_last_line() {
            self.cursor.x = self.current_line_len();
        } else {
            self.cursor.y += 1;
            self.cursor.x = std::cmp::min(self.cursor.x % width, self.current_line_len());
        }

        self
    }

    pub fn end_of_line(mut self) -> Document {
        self.cursor.x = self.current_line_len();

        self
    }

    pub fn start_of_line(mut self) -> Document {
        self.cursor.x = 0;

        self
    }
}
