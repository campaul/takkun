use std::io;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::panic;
use std::sync::mpsc;
use std::thread;

// TODO: use libc instead of termios
use termios::*;

pub const HIDE_CURSOR: &[u8; 6] = b"\x1b[?25l";
pub const SHOW_CURSOR: &[u8; 6] = b"\x1b[?25h";
pub const ZERO_CURSOR: &[u8; 3] = b"\x1b[H";
pub const CLEAR_LINE: &[u8; 3] = b"\x1b[K";

macro_rules! position_cursor {
    ($c:expr) => {
        format!("\x1b[{};{}H", $c.y + 1, $c.x + 1).as_str().bytes()
    };
}

lazy_static! {
    // TODO: error handling
    static ref TERMIOS: termios::Termios = Termios::from_fd(
        io::stdout().as_raw_fd()
    ).unwrap();

    static ref PIPES: [i32; 2] = (|| {
        let mut fds = [0; 2];

        unsafe {
            // TODO: error handling
            libc::pipe(fds.as_mut_ptr());
        }

        fds
    })();
}

pub enum Event {
    Up,
    Down,
    Left,
    Right,

    PageUp,
    PageDown,

    Home,
    End,

    Delete,
    Escape,

    Nothing,

    Resize(usize, usize),

    Exit,
    Error(String),
}

fn ctrl(k: char) -> char {
    (k as u8 & 0x1f) as char
}

fn read_char(stdin: &mut io::Stdin) -> io::Result<char> {
    let mut buffer: [u8; 1] = [0];
    stdin.read_exact(&mut buffer)?;
    Ok(buffer[0] as char)
}

fn parse_tilda(stdin: &mut io::Stdin, event: Event) -> Event {
    match read_char(stdin) {
        Ok('~') => event,
        _ => Event::Escape,
    }
}

fn parse_bracket(stdin: &mut io::Stdin) -> Event {
    match read_char(stdin) {
        Ok(c) => match c {
            '1' => parse_tilda(stdin, Event::Home),
            '3' => parse_tilda(stdin, Event::Delete),
            '4' => parse_tilda(stdin, Event::End),
            '5' => parse_tilda(stdin, Event::PageUp),
            '6' => parse_tilda(stdin, Event::PageDown),
            '7' => parse_tilda(stdin, Event::Home),
            '8' => parse_tilda(stdin, Event::End),
            'A' => Event::Up,
            'B' => Event::Down,
            'C' => Event::Right,
            'D' => Event::Left,
            'H' => Event::Home,
            'F' => Event::End,
            _ => Event::Escape,
        },
        _ => Event::Escape,
    }
}

fn parse_o(stdin: &mut io::Stdin) -> Event {
    match read_char(stdin) {
        Ok(c) => match c {
            'H' => Event::Home,
            'F' => Event::End,
            _ => Event::Escape,
        },
        _ => Event::Escape,
    }
}

fn parse_escape(stdin: &mut io::Stdin) -> Event {
    match read_char(stdin) {
        Ok(c) => match c {
            '[' => parse_bracket(stdin),
            'O' => parse_o(stdin),
            _ => Event::Escape,
        },
        _ => Event::Escape,
    }
}

fn process_keypress() -> Event {
    let mut stdin = io::stdin();

    loop {
        match read_char(&mut stdin) {
            Ok(c) => {
                if c == '\x1b' {
                    return parse_escape(&mut stdin);
                }

                if c == ctrl('q') {
                    return Event::Exit;
                }

                match c as char {
                    'w' => return Event::Up,
                    'a' => return Event::Left,
                    's' => return Event::Down,
                    'd' => return Event::Right,
                    _ => {}
                }

                return Event::Nothing;
            }
            Err(e) => match e.kind() {
                ErrorKind::UnexpectedEof => {}
                _ => return Event::Error(e.to_string()),
            },
        }
    }
}

pub fn raw_mode_termios(termios: &Termios) -> Termios {
    let mut raw_termios = termios.clone();

    raw_termios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
    raw_termios.c_oflag &= !(OPOST);
    raw_termios.c_cflag |= CS8;
    raw_termios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
    raw_termios.c_cc[VMIN] = 0;
    raw_termios.c_cc[VTIME] = 1;

    raw_termios
}

pub fn get_window_size() -> io::Result<Event> {
    let stdout = io::stdout();

    let size = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    unsafe {
        let status = libc::ioctl(stdout.as_raw_fd(), libc::TIOCGWINSZ, &size);

        if status == -1 {
            return Err(io::Error::new(
                ErrorKind::Other,
                "Error reading terminal size.",
            ));
        }
    }

    Ok(Event::Resize(size.ws_col as usize, size.ws_row as usize))
}

pub fn handle_resize() {
    let write = PIPES[1];

    let buff: [u8; 1] = [0];

    unsafe {
        libc::write(write, buff.as_ptr() as *mut libc::c_void, 1);
    }
}

fn write(buffer: &[u8]) -> io::Result<()> {
    // TODO: verify the following
    // - we are in raw mode
    // - buffer matches screen size
    // - buffer doesn't have anything we can't or don't want to display
    let mut stdout = io::stdout();
    stdout.write_all(buffer)?;
    stdout.flush()?;
    Ok(())
}

pub type In = impl Fn() -> Event;
pub type Out = impl Fn(&[u8]) -> io::Result<()>;

pub fn enter_raw_mode() -> io::Result<(In, Out)> {
    let stdout = io::stdout();

    tcsetattr(stdout.as_raw_fd(), TCSAFLUSH, &raw_mode_termios(&TERMIOS))?;

    let default_panic_hook = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        exit_raw_mode().unwrap();
        default_panic_hook(info);
    }));

    let (tx, rx) = mpsc::channel::<Event>();

    let stdin_tx = tx.clone();
    let signal_tx = tx.clone();

    thread::spawn(move || loop {
        stdin_tx.send(process_keypress()).unwrap();
    });

    thread::spawn(move || {
        let read = PIPES[0];

        loop {
            let buff: [u8; 1] = [0];
            // TODO: error handling
            unsafe {
                libc::read(read, buff.as_ptr() as *mut libc::c_void, 1);
            }

            let size = get_window_size().unwrap();
            signal_tx.send(size).unwrap();
        }
    });

    unsafe {
        libc::signal(libc::SIGWINCH, handle_resize as libc::sighandler_t);
    }

    let size = get_window_size().unwrap();
    tx.send(size).unwrap();

    let read = move || match rx.recv() {
        Ok(e) => e,
        Err(e) => Event::Error(e.to_string()),
    };

    Ok((read, write))
}

pub fn exit_raw_mode() -> io::Result<()> {
    let mut stdout = io::stdout();

    stdout.write_all(b"\x1b[2J")?;
    stdout.write_all(b"\x1b[H")?;

    tcsetattr(stdout.as_raw_fd(), TCSAFLUSH, &TERMIOS)?;

    stdout.flush()?;

    Ok(())
}
