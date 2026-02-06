use crate::tty_echo_serialport::echo;

pub mod find_tty_serialport;
pub mod tty_echo_serialport;
pub mod read_file;
pub mod generate_crc;
pub mod test_echo;
pub mod test_crc;
pub mod boot;
pub mod parse_invocation;
pub mod test_send;

fn main() {
    let status = boot::full_boot();

    if let Err(error) = status {
        print!("Error: {}\n", error);
    }
}
