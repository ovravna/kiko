use std::io::{stdin, stdout, stderr, Read, Write};
use std::io;
use termios::*;
use ascii; 

// macro_rules! ctrl_key {

//     ( $e:expr ) => { 
//         ($e  & 0x1f) as char
//     }
// }

const STDIN_FD: i32 = 0;

struct RawMode {
    termios: Termios
}

impl Drop for RawMode {
    fn drop(&mut self) {
        disable_raw_mode(self.termios);
    }
}

fn editor_read_key(reader: &mut impl Read) -> char {
    let mut res = [0];
    let mut nread = 0;
    while nread <= 0 {
        nread = reader.read(&mut res)
            .expect("read");
    }
    return res[0] as char;
}

fn editor_process_keypress() -> bool {

    let c = editor_read_key(&mut stdin());

    if c == ctrl_key('q') {
        return false;
    }

    return true;
}

fn editor_refresh_screen() -> Result<(), io::Error> {
    stdout().write(b"\x1b[2J")?;
    stdout().flush()?;
    Ok(())
}

fn is_ctrl(c: char) -> bool {
    return match ascii::AsciiChar::from(c) {
        Ok(ch) => ch.is_control(),
        Err(_) => false
    }; 
}

#[test]
fn test_is_ctrl() {
    assert!(!is_ctrl('a'));
    assert!(!is_ctrl('9'));
    assert!(is_ctrl(7 as char));
}

fn ctrl_key(c: char) -> char {
    (c as u8 & 0x1f) as char
}

#[test]
fn test_ctrl_key() {
    assert_eq!(ctrl_key('j'), 10 as char);
    assert_eq!(ctrl_key('a'), 1 as char);
}

fn run() {

   loop {
       editor_refresh_screen().expect("refresh");

       if !editor_process_keypress() {
           break; 
       }

   }
}

fn disable_raw_mode(termios: Termios) {
    tcsetattr(STDIN_FD, TCSAFLUSH, &termios).expect("tcsetattr");
}
fn enable_raw_mode() -> RawMode {

    let mut raw = Termios::from_fd(STDIN_FD).expect("tcgetattr");
    tcgetattr(STDIN_FD, &mut raw).expect("tcsetattr");

    let rm = RawMode { termios: raw };
    raw.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
    raw.c_oflag &= !(OPOST);
    raw.c_cflag |= CS8;
    raw.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
    raw.c_cc[VMIN] = 0;
    raw.c_cc[VTIME] = 1;

    tcsetattr(STDIN_FD, TCSAFLUSH, &mut raw).expect("tcsetattr");

    return rm;
}

fn main() {
    let _rm = enable_raw_mode(); 
    run()
}
