use std::{io, thread, time};
use crate::find_tty_serialport::find_tty;

pub fn test_send() -> io::Result<()> {
    let mut port = find_tty()?;
    
    /*
     * Stall 1 sec
     */
    let one_sec = time::Duration::from_secs(1);
    thread::sleep(one_sec);

    let buffer : [u8 ; 3] = [0x67, 0x68, 0x69];
    port.write_all(&buffer);
    port.flush();

    Ok(())
}