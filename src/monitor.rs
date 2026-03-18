mod find_tty_serialport;

use find_tty_serialport::find_tty;
use std::io::{self, Read, Write};
use std::thread;

fn main() -> io::Result<()> {
    let mut port = find_tty()?;
    println!("Serial monitor started. Press Ctrl-C to quit.");

    /*
     * Spawn a thread to forward stdin to the serial port.
     */
    let mut port_writer = port.try_clone()?;
    thread::spawn(move || {
        let mut buf = [0u8; 1];
        loop {
            if io::stdin().read(&mut buf).unwrap_or(0) > 0 {
                let _ = port_writer.write_all(&buf);
                let _ = port_writer.flush();
            }
        }
    });

    /*
     * Main thread reads from serial and prints to stdout.
     */
    let mut buf = [0u8; 256];
    loop {
        match port.read(&mut buf) {
            Ok(n) if n > 0 => {
                io::stdout().write_all(&buf[..n])?;
                io::stdout().flush()?;
            }
            Ok(_) => {}
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
            Err(e) => return Err(e),
        }
    }
}
