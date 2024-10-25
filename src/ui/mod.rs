use std::io;

use crate::document::Cursor;
use crate::document::Document;
use crate::terminal::Event;

mod file_chooser;
mod find;
mod status;
mod tabs;
mod text_area;

pub use file_chooser::FileChooser;
pub use find::Find;
pub use status::Status;
pub use tabs::Tabs;
pub use text_area::TextArea;

pub struct Window {
    pub lines: Vec<String>,
    pub cursor: Cursor,
}

pub trait Component {
    fn update(&mut self, e: Event, width: usize) -> io::Result<bool>;
    fn render(&mut self, width: usize, height: usize) -> Window;
    fn document(&mut self) -> &mut Document;
}
