use std::io;
use crate::{find_tty_serialport::find_tty, tty_echo_serialport};

pub fn test_echo() {
    let tty = find_tty();
    let res : Result<(), io::Error> = tty_echo_serialport::echo(tty.unwrap());
    
    if let Err(e) = res {
        eprintln!("Error: {}", e);
    }
}