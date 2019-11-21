use std::io::{stdin, Read};
use termios::*;

const STDIN_FD: i32 = 0;

struct RawMode {
    termios: Termios
}
impl Drop for RawMode {
    fn drop(&mut self) {
        disable_raw_mode(self.termios);
    }
}

fn run() {

   let mut res = [0];
   while let Ok(_) = stdin().read(&mut res) {

       if res[0] == 'q' as u8{
           break;
       }

       println!("{}", res[0]);
   }
}

fn disable_raw_mode(termios: Termios) {
    tcsetattr(STDIN_FD, TCSAFLUSH, &termios).unwrap();
}
fn enable_raw_mode() -> RawMode {

    let mut raw = Termios::from_fd(STDIN_FD).unwrap();

    tcgetattr(STDIN_FD, &mut raw).unwrap();
    raw.c_lflag &= !ECHO;

    tcsetattr(STDIN_FD, TCSAFLUSH, &mut raw).unwrap();

    return RawMode { termios: raw }
}

fn main() {
    let _rm = enable_raw_mode(); 
    run()
}
