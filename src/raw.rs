use std::io;

use std::os::unix::io::RawFd;
use termios::*;

fn raw_mode_termios(termios: &Termios) -> Termios {
    let mut raw_termios = termios.clone();

    raw_termios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
    raw_termios.c_oflag &= !(OPOST);
    raw_termios.c_cflag |= CS8;
    raw_termios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
    raw_termios.c_cc[VMIN] = 0;
    raw_termios.c_cc[VTIME] = 1;

    raw_termios
}

pub fn in_raw_mode(fd: RawFd, f: &mut dyn FnMut() -> io::Result<()>) -> io::Result<()> {
    let termios = Termios::from_fd(fd)?;

    tcsetattr(fd, TCSAFLUSH, &raw_mode_termios(&termios))?;
    f()?;
    tcsetattr(fd, TCSAFLUSH, &termios)?;

    Ok(())
}
