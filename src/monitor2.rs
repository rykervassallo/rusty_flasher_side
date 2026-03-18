mod find_tty_serialport;

use std::io::{self, Read, Write};
use std::thread;
use std::time::Duration;

fn find_available_tty() -> io::Result<Box<dyn serialport::SerialPort>> {
    let ports = serialport::available_ports()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    for port in &ports {
        if port.port_name.contains("tty.usb") {
            println!("Trying port: {}", port.port_name);
            match serialport::new(&port.port_name, 115200)
                .timeout(Duration::from_millis(100))
                .open()
            {
                Ok(p) => {
                    println!("Opened port: {}", port.port_name);
                    return Ok(p);
                }
                Err(e) => {
                    println!("Port {} busy ({}), trying next...", port.port_name, e);
                }
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "No available TTY USB device found",
    ))
}

fn main() -> io::Result<()> {
    let mut port = find_available_tty()?;
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
