use std::io::{stdin, stdout, Read, Write};
use std::io;
use std::iter;
use termios::*;
use ascii; 

#[macro_use] extern crate scan_fmt;

macro_rules! io {
    ($t:ty) => { Result<$t, io::Error> }
}

const STDIN_FD: i32 = 0;

struct EditorConfig {
    height: usize,
    width: usize,
    orig_termios: Termios
}

impl Drop for EditorConfig {
    fn drop(&mut self) {
        disable_raw_mode(self.orig_termios).expect("fail");
    }
}

fn editor_read_key(reader: &mut impl Read) -> io!(char) {
    let mut res = [0];
    let mut nread = 0;
    while nread <= 0 {
        nread = reader.read(&mut res)?;
    }
    return Ok(res[0] as char);
}

fn editor_process_keypress() -> io!(bool) {

    let c = editor_read_key(&mut stdin())?;

    if c == ctrl_key('q') {
        editor_clean_screen(&mut stdout())?;
        return Ok(false);
    }

    return Ok(true);
}


fn editor_clean_screen(out: &mut impl Write) -> io!(()) {
    out.write(b"\x1b[2J")?;
    out.write(b"\x1b[H")?;

    return Ok(());
}

fn editor_refresh_screen(out: &mut impl Write, conf: &EditorConfig) -> io!(()) {

    out.write(b"\x1b[?25l")?;
    out.write(b"\x1b[H")?;

    editor_draw_rows(out, conf)?;

    out.write(b"\x1b[H")?;
    out.write(b"\x1b[?25h")?;
    

    Ok(())
}

fn editor_draw_rows(out: &mut impl Write, conf: &EditorConfig) -> io!(()) {

    for y in 0..conf.height {
        if y == conf.height / 3 {
            let text = format!("Rilo Editor -- the finest");
            let w = std::cmp::min(text.len(), conf.width);

            let mut space: Vec<u8> = iter::repeat(b' ')
                .take((conf.width - w) / 2)
                .collect(); 

            space[0] = b'~';

            out.write(&space)?;
            out.write(text[..w].as_bytes())?;
            

        } 
        else {
            out.write(b"~")?;
        }
        out.write(b"\x1b[0K")?;
        
        if y != conf.height - 1 {
            out.write(b"\r\n")?;
        }

    }
    return Ok(());
}

fn get_cursor_position() -> io!((usize, usize)) {
    stdout().write(b"\x1b[6n")?;
    stdout().flush()?;

    let mut buf: [u8; 32] = [0;32];
    stdin().read(&mut buf)?;
    
    let s = std::str::from_utf8(&buf).expect("str");
    println!("{}", s);

    let dim = match scan_fmt!(s, "\x1b[{};{}R", usize, usize) {
        Ok(d) => d,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("{}", e)))
    };  
    
    return Ok(dim);
}

fn set_cursor_position(x: usize, y: usize) -> io!(()) {

    let val = format!("\x1b[{}C\x1b[{}B", x, y);

    stdout().write(val.as_bytes())?;

    return Ok(());
}

fn get_window_size() -> io!((usize, usize)) {

    // let dim = term_size::dimensions().expect("faak");

    let (y, x) = get_cursor_position()?;

    set_cursor_position(999, 999)?;

    let dim = get_cursor_position()?;
    
    set_cursor_position(x, y)?;

    return Ok(dim);
}

fn editor_init(termios: Termios) -> io!(EditorConfig) {

    let (h, w) = get_window_size()?;

    let conf = EditorConfig {
        height: h,
        width: w,
        orig_termios: termios
    };

    return Ok(conf);
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

fn run() -> io!(()) {

    

   let termios = enable_raw_mode()?;
   let conf = editor_init(termios)?;
   let mut out = stdout(); //io::BufWriter::new(stdout());
    
   
   loop {
       editor_refresh_screen(&mut out, &conf)?;
       out.flush()?;
       

       if !editor_process_keypress()? {
           break; 
       }
   }

   return Ok(());
}

fn disable_raw_mode(termios: Termios) -> io!(()){
    tcsetattr(STDIN_FD, TCSAFLUSH, &termios)?;
    return Ok(()); 
}
fn enable_raw_mode() -> io!(Termios) {

    let mut raw = Termios::from_fd(STDIN_FD)?;
    tcgetattr(STDIN_FD, &mut raw)?;

    let orig = raw; 
    // let rm = EditorConfig { orig_termios: raw };

    raw.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
    raw.c_oflag &= !(OPOST);
    raw.c_cflag |= CS8;
    raw.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
    raw.c_cc[VMIN] = 0;
    raw.c_cc[VTIME] = 1;

    tcsetattr(STDIN_FD, TCSAFLUSH, &mut raw)?;

    return Ok(orig);
}

fn main() {
    match run() {
        Ok(_) => {},
        Err(err) => {
            editor_clean_screen(&mut stdout()).expect("fail");
            panic!(err);
        }
    }
}
