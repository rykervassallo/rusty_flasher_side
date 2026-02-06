use std::io;
use crate::{find_tty_serialport::find_tty, tty_echo_serialport};

pub fn test_echo() -> io::Result<()> {
    let mut tty = find_tty()?;
    tty_echo_serialport::echo(&mut tty)?;
    Ok(())
}