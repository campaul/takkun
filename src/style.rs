#[derive(Clone, PartialEq)]
pub enum Decoration {
    Italic,
    Underline,
}

#[derive(Clone, PartialEq)]
pub struct Style {
    pub foreground: u8,
    pub background: u8,
    pub decoration: Vec<Decoration>,
}

fn decoration(style: &Style) -> String {
    let mut decorations = String::new();

    for d in &style.decoration {
        match d {
            Decoration::Italic => decorations.push_str("\x1b[3m"),
            Decoration::Underline => decorations.push_str("\x1b[4m"),
        }
    }

    decorations
}

pub fn tabbed(text: &String) -> String {
    text.replace("\t", std::str::from_utf8(&[b' '; 4]).unwrap())
}

pub fn styled(style: &Style, text: &String) -> String {
    format!(
        "\x1b[0m{}\x1b[38;5;{}m\x1b[48;5;{}m{}",
        decoration(&style),
        style.foreground,
        style.background,
        text,
    )
}
