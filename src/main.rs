use std::io;
use std::io::ErrorKind;
use std::io::Read;
use std::os::unix::io::AsRawFd;

mod raw;

fn is_control(c: u8) -> bool {
    return c < 32 || c == 127;
}

fn main() -> io::Result<()> {
    let mut stdin = io::stdin();
    let stdin_fd = stdin.as_raw_fd();

    let mut buffer: [u8; 1] = [0];

    raw::in_raw_mode(stdin_fd, &mut || {
        loop {
            buffer[0] = 0;

            match stdin.read_exact(&mut buffer) {
                Ok(..) => {
                    let c = buffer[0];

                    if c == 'q' as u8 {
                        break;
                    }

                    if !is_control(c) {
                        println!("{}\r", c as char);
                    }
                }
                Err(e) => match e.kind() {
                    ErrorKind::UnexpectedEof => {}
                    _ => panic!("{:?}", e),
                },
            }
        }

        Ok(())
    })
}
