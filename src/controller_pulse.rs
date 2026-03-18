mod find_tty_serialport;

use find_tty_serialport::find_tty;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use device_query::{DeviceQuery, DeviceState, Keycode};
use crossterm::terminal;

const TURN_LEFT_COMMAND  : u8 = 0x01;
const TURN_RIGHT_COMMAND : u8 = 0x02;
const FORWARD_COMMAND    : u8 = 0x03;
const BACKWARD_COMMAND   : u8 = 0x04;

const POLL_INTERVAL_MS : u64 = 10;
const BURST_DURATION_MS : u64 = 200;

fn send_command(port: &mut Box<dyn serialport::SerialPort>, cmd: u8) -> io::Result<()> {
    port.write_all(&[cmd])?;
    port.flush()?;
    Ok(())
}

fn send_burst(port: &mut Box<dyn serialport::SerialPort>, cmd: u8) -> io::Result<()> {
    let iterations = BURST_DURATION_MS / POLL_INTERVAL_MS;
    for _ in 0..iterations {
        send_command(port, cmd)?;
        thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let mut port = find_tty()?;
    println!("Pulse controller. Arrow keys send commands every poll. Press 'q' to quit.");

    terminal::enable_raw_mode()?;

    let device_state = DeviceState::new();
    let mut g_held = false;
    let mut h_held = false;

    loop {
        let keys = device_state.get_keys();

        if keys.contains(&Keycode::Q) || keys.contains(&Keycode::Escape) {
            break;
        }

        if keys.contains(&Keycode::G) {
            if !g_held {
                g_held = true;
                send_burst(&mut port, TURN_LEFT_COMMAND)?;
            }
        } else {
            g_held = false;
        }

        if keys.contains(&Keycode::H) {
            if !h_held {
                h_held = true;
                send_burst(&mut port, TURN_RIGHT_COMMAND)?;
            }
        } else {
            h_held = false;
        }

        if keys.contains(&Keycode::Up) {
            send_command(&mut port, FORWARD_COMMAND)?;
        } else if keys.contains(&Keycode::Down) {
            send_command(&mut port, BACKWARD_COMMAND)?;
        } else if keys.contains(&Keycode::Left) {
            send_command(&mut port, TURN_LEFT_COMMAND)?;
        } else if keys.contains(&Keycode::Right) {
            send_command(&mut port, TURN_RIGHT_COMMAND)?;
        }

        thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
    }

    terminal::disable_raw_mode()?;
    println!("Exiting controller.");
    Ok(())
}
