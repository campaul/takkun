use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

#[derive(Copy, Clone)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
}

pub struct Document {
    pub rows: Vec<String>,
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

        self.rows = contents.lines().map(String::from).collect();
        self.cursor = Cursor { x: 0, y: 0 };
        self.filename = Some(filename);

        Ok(())
    }

    pub fn name(&self) -> String {
        self.filename.clone().unwrap_or("New File".to_string())
    }

    pub fn insert(&mut self, c: String) {
        assert!(c.len() == 1);

        if self.rows.len() == 0 {
            self.rows.push(String::new());
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

        self.rows.insert(self.cursor.y, last.to_string());
        self.rows.insert(self.cursor.y, first.to_string());

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

            self.rows[self.cursor.y] = format!("{}{}", self.rows[self.cursor.y], prev);
        } else if !self.on_first_char() {
            self.cursor.x -= 1;
            self.rows[self.cursor.y].remove(self.cursor.x);
        }
    }

    pub fn tab(&mut self) {
        for _ in 0..4 {
            self.insert(" ".to_string());
        }
    }

    pub fn set_filename(&mut self, filename: String) {
        self.filename = Some(filename);
    }

    pub fn save(&self) -> std::io::Result<()> {
        if let Some(filename) = &self.filename {
            let mut buffer = File::create(filename)?;

            for row in &self.rows {
                buffer.write_all(row.as_bytes())?;
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
                matches.push((m.0, i));
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
}
