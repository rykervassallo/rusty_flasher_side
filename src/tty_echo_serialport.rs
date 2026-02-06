use std::io::{self, Read, Write};
use serialport::SerialPort;

pub fn echo(port : &mut Box<dyn SerialPort>) -> io::Result<()> {
    let mut stdout : io::Stdout = io::stdout();
    println!("TTY opened, printing ingress to terminal…");
    let mut buf: [u8; 1024] = [0u8; 1024];

    loop {
        match port.read(&mut buf) {
            /*
             * Reading bytes.
             */
            Ok(n) if n > 0 => {
                stdout.write_all(&buf[..n])?;
                // for &b in &buf[..n] {
                //     write!(stdout, "{:02x}", b)?;
                // }
                stdout.flush()?;
            }

            /*
             * Reading nothing (successfully).
             */
            Ok(_) => {
                continue;
            }

            /*
             * Timeout.
             */
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                continue;
            }

            /*
             * Other error of some kind.
             */
            Err(e) => {
                return Err(e);
            }
        }
    }
}