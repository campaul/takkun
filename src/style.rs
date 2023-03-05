pub struct Style {
    pub foreground: u8,
    pub background: u8,
}

pub fn styled(style: Style, text: String) -> String {
    format!(
        "\x1b[38;5;{}m\x1b[48;5;{}m{}\x1b[0m",
        style.foreground, style.background, text
    )
}
