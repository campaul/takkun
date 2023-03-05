use std::io;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::panic;
use std::sync::mpsc;
use std::thread;

pub const HIDE_CURSOR: &[u8; 6] = b"\x1b[?25l";
pub const SHOW_CURSOR: &[u8; 6] = b"\x1b[?25h";
pub const ZERO_CURSOR: &[u8; 3] = b"\x1b[H";
pub const CLEAR_LINE: &[u8; 3] = b"\x1b[K";

macro_rules! position_cursor {
    ($c:expr) => {
        format!("\x1b[{};{}H", $c.y + 1, $c.x + 1).as_str().bytes()
    };
}

static mut PIPES: [i32; 2] = [0; 2];
static mut TERMIOS: libc::termios = libc::termios {
    c_iflag: 0,
    c_oflag: 0,
    c_cflag: 0,
    c_lflag: 0,
    c_cc: [0; 32],
    c_ispeed: 0,
    c_ospeed: 0,
    c_line: 0,
};

pub enum Event {
    Input(String),
    Control(String),

    Up,
    Down,
    Left,
    Right,

    PageUp,
    PageDown,

    Home,
    End,

    Tab,
    Delete,
    Backspace,
    Escape,
    Enter,

    Nothing,

    Pause,
    Resume,

    Resize,

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

    match read_char(&mut stdin) {
        Ok(c) => {
            if c == '\x1b' {
                return parse_escape(&mut stdin);
            }

            if c == ctrl('q') {
                return Event::Control("q".to_string());
            }

            if c == ctrl('o') {
                return Event::Control("o".to_string());
            }

            if c == ctrl('f') {
                return Event::Control("f".to_string());
            }

            if c == ctrl('z') {
                return Event::Pause;
            }

            if c == 13 as char {
                return Event::Enter;
            }

            if c == 127 as char {
                return Event::Backspace;
            }

            if (c as u8) == 9 {
                return Event::Tab;
            }

            if (c as u8) > 31 && (c as u8) < 127 {
                return Event::Input(c.to_string());
            }

            return Event::Nothing;
        }

        Err(e) => match e.kind() {
            ErrorKind::UnexpectedEof => Event::Nothing,
            _ => return Event::Error(e.to_string()),
        },
    }
}

pub fn raw_mode_termios(termios: &libc::termios) -> libc::termios {
    let mut raw_termios = termios.clone();

    raw_termios.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
    raw_termios.c_oflag &= !(libc::OPOST);
    raw_termios.c_cflag |= libc::CS8;
    raw_termios.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);
    raw_termios.c_cc[libc::VMIN] = 0;
    raw_termios.c_cc[libc::VTIME] = 1;

    raw_termios
}

pub fn get_window_size() -> io::Result<(usize, usize)> {
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

    Ok((size.ws_col as usize, size.ws_row as usize))
}

#[repr(u8)]
pub enum Signal {
    SIGWINCH,
    SIGCONT,
}

pub fn handle_signal(signal: Signal) {
    unsafe {
        libc::write(PIPES[1], [signal].as_ptr() as *mut libc::c_void, 1);
    }
}

fn handle_resize() {
    handle_signal(Signal::SIGWINCH);
}

fn handle_cont() {
    handle_signal(Signal::SIGCONT);
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

pub fn enter_alternate_buffer() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(b"\x1b[?1049h\033[2J\x1b[H")?;
    stdout.flush()?;
    Ok(())
}

pub fn exit_alternate_buffer() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(b"\x1b[2J\x1b[H\x1b[?1049l")?;
    stdout.flush()?;
    Ok(())
}

pub fn init() -> io::Result<(In, Out)> {
    let stdout = io::stdout();

    unsafe {
        libc::tcgetattr(stdout.as_raw_fd(), &mut TERMIOS);
        libc::pipe(PIPES.as_mut_ptr());
    }

    enter_alternate_buffer()?;
    enter_raw_mode()?;

    let default_panic_hook = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        exit().unwrap();
        default_panic_hook(info);
    }));

    let (tx, rx) = mpsc::channel::<Event>();

    let stdin_tx = tx.clone();
    let signal_tx = tx.clone();

    thread::spawn(move || loop {
        stdin_tx.send(process_keypress()).unwrap();
    });

    thread::spawn(move || {
        loop {
            let buf: [u8; 1] = [0];
            // TODO: error handling
            unsafe {
                libc::read(PIPES[0], buf.as_ptr() as *mut libc::c_void, 1);
            }

            let s: Signal = unsafe { std::mem::transmute(buf[0]) };

            match s.try_into() {
                Ok(Signal::SIGWINCH) => {
                    signal_tx.send(Event::Resize).unwrap();
                }
                Ok(Signal::SIGCONT) => {
                    signal_tx.send(Event::Resume).unwrap();
                    signal_tx.send(Event::Resize).unwrap();
                }
                _ => {}
            }
        }
    });

    unsafe {
        libc::signal(libc::SIGWINCH, handle_resize as libc::sighandler_t);
        libc::signal(libc::SIGCONT, handle_cont as libc::sighandler_t);
    }

    tx.send(Event::Resize).unwrap();

    let read = move || match rx.recv() {
        Ok(e) => e,
        Err(e) => Event::Error(e.to_string()),
    };

    Ok((read, write))
}

pub fn exit() -> io::Result<()> {
    exit_raw_mode()?;
    exit_alternate_buffer()?;

    Ok(())
}

pub fn enter_raw_mode() -> io::Result<()> {
    let stdout = io::stdout();

    unsafe {
        // TODO: error handling
        libc::tcsetattr(
            stdout.as_raw_fd(),
            libc::TCSAFLUSH,
            &raw_mode_termios(&TERMIOS),
        );
    }

    Ok(())
}

pub fn exit_raw_mode() -> io::Result<()> {
    let stdout = io::stdout();

    unsafe {
        // TODO: error handling
        libc::tcsetattr(stdout.as_raw_fd(), libc::TCSAFLUSH, &TERMIOS);
    }

    Ok(())
}

pub fn pause() -> io::Result<()> {
    exit_raw_mode().unwrap();
    exit_alternate_buffer().unwrap();

    unsafe {
        libc::kill(std::process::id() as i32, libc::SIGTSTP);
    }

    Ok(())
}

pub fn resume() -> io::Result<()> {
    enter_alternate_buffer()?;
    enter_raw_mode()?;

    Ok(())
}
