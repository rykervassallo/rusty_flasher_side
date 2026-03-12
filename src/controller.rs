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
const STOP_COMMAND       : u8 = 0x05;
const ACK                : u8 = 0x06;

const POLL_INTERVAL_MS : u64 = 10;
const PULSE_DURATION_MS : u64 = 160;


fn send_command(port: &mut Box<dyn serialport::SerialPort>, cmd: u8) -> io::Result<()> {
    port.write_all(&[cmd])?;
    port.flush()?;
    Ok(())
}

fn arrow_to_command(key: &Keycode) -> Option<(u8, &'static str)> {
    match key {
        Keycode::Up    => Some((FORWARD_COMMAND,    "FORWARD")),
        Keycode::Down  => Some((BACKWARD_COMMAND,   "BACKWARD")),
        Keycode::Left  => Some((TURN_LEFT_COMMAND,  "TURN LEFT")),
        Keycode::Right => Some((TURN_RIGHT_COMMAND, "TURN RIGHT")),
        _ => None,
    }
}

fn main() -> io::Result<()> {
    let mut port = find_tty()?;
    println!("Connected to serial port. Use arrow keys to control. Press 'q' to quit.");

    /*
     * Raw mode stops the terminal from echoing escape sequences.
     */
    terminal::enable_raw_mode()?;

    let device_state = DeviceState::new();
    let mut active_key: Option<Keycode> = None;
    let mut g_held = false;
    let mut h_held = false;

    loop {
        let keys = device_state.get_keys();

        /*
         * Check for quit.
         */
        if keys.contains(&Keycode::Q) || keys.contains(&Keycode::Escape) {
            break;
        }

        /*
         * G: timed right turn pulse (fire once per press).
         */
        if keys.contains(&Keycode::G) {
            if !g_held {
                g_held = true;
                print!("> PULSE TURN RIGHT\r\n");
                send_command(&mut port, TURN_RIGHT_COMMAND)?;
                thread::sleep(Duration::from_millis(PULSE_DURATION_MS));
                send_command(&mut port, STOP_COMMAND)?;
                print!("> STOP\r\n");
            }
        } else {
            g_held = false;
        }

        /*
         * H: timed forward pulse (fire once per press).
         */
        if keys.contains(&Keycode::H) {
            if !h_held {
                h_held = true;
                print!("> PULSE FORWARD\r\n");
                send_command(&mut port, FORWARD_COMMAND)?;
                thread::sleep(Duration::from_millis(PULSE_DURATION_MS));
                send_command(&mut port, STOP_COMMAND)?;
                print!("> STOP\r\n");
            }
        } else {
            h_held = false;
        }

        /*
         * Find the first arrow key currently held.
         */
        let current_arrow = keys.iter().find_map(|k| {
            arrow_to_command(k).map(|_| k.clone())
        });

        match (&active_key, &current_arrow) {
            /*
             * No arrow was held, and one is now pressed.
             */
            (None, Some(key)) => {
                let (cmd, label) = arrow_to_command(key).unwrap();
                print!("> {}\r\n", label);
                send_command(&mut port, cmd)?;
                active_key = Some(key.clone());
            }

            /*
             * An arrow was held, and a different one is now pressed.
             */
            (Some(prev), Some(key)) if prev != key => {
                let (cmd, label) = arrow_to_command(key).unwrap();
                print!("> {}\r\n", label);
                send_command(&mut port, cmd)?;
                active_key = Some(key.clone());
            }

            /*
             * An arrow was held, and it has now been released.
             */
            (Some(_), None) => {
                print!("> STOP\r\n");
                send_command(&mut port, STOP_COMMAND)?;
                active_key = None;
            }

            /*
             * Same key still held, or nothing happening.
             */
            _ => {}
        }

        thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
    }

    terminal::disable_raw_mode()?;
    println!("Exiting controller.");
    Ok(())
}
