use serialport::{self, SerialPort};
use std::io;
use std::time::Duration;

pub fn find_tty() -> io::Result<Box<dyn SerialPort>> {
    /*
     * Grab available ports.
     */
    let ports = serialport::available_ports()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    /*
     * Loop through and look for usb connection.
     */
    for port in ports {
        if port.port_name.contains("tty.usb") {
            println!("Found port: {}", port.port_name);
            
            return serialport::new(&port.port_name, 115200) 
                .timeout(Duration::from_millis(100))
                .open()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e));
        }
    }
    
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "No TTY USB device found",
    ))
}