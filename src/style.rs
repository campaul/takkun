pub enum Decoration {
    Italic,
    Underline,
}

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

pub fn styled(style: &Style, text: &String) -> String {
    format!(
        "\x1b[0m{}\x1b[38;5;{}m\x1b[48;5;{}m{}\x1b[0m",
        decoration(&style),
        style.foreground,
        style.background,
        text
    )
}
