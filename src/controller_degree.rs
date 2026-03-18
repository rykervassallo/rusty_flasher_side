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

fn send_command(port: &mut Box<dyn serialport::SerialPort>, cmd: u8, amount: u8) -> io::Result<()> {
    port.write_all(&[cmd, amount])?;
    port.flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    let mut port = find_tty()?;
    println!("Degree controller. Arrows=15deg, G/H=30deg. Press 'q' to quit.");

    terminal::enable_raw_mode()?;

    let device_state = DeviceState::new();
    let mut left_held = false;
    let mut right_held = false;
    let mut g_held = false;
    let mut h_held = false;
    let mut up_held = false;
    let mut down_held = false;
    let mut x_held = false;
    let mut c_held = false;
    let mut v_held = false;
    let mut b_held = false;
    let mut n_held = false;
    let mut m_held = false;

    loop {
        let keys = device_state.get_keys();

        if keys.contains(&Keycode::Q) || keys.contains(&Keycode::Escape) {
            break;
        }

        /*
         * G: 30 degree left turn (once per press).
         */
        if keys.contains(&Keycode::G) {
            if !g_held {
                g_held = true;
                send_command(&mut port, TURN_LEFT_COMMAND, 30)?;
            }
        } else {
            g_held = false;
        }

        /*
         * H: 30 degree right turn (once per press).
         */
        if keys.contains(&Keycode::H) {
            if !h_held {
                h_held = true;
                send_command(&mut port, TURN_RIGHT_COMMAND, 30)?;
            }
        } else {
            h_held = false;
        }

        /*
         * Left arrow: 15 degree left turn (once per press).
         */
        if keys.contains(&Keycode::Left) {
            if !left_held {
                left_held = true;
                send_command(&mut port, TURN_LEFT_COMMAND, 15)?;
            }
        } else {
            left_held = false;
        }

        /*
         * Right arrow: 15 degree right turn (once per press).
         */
        if keys.contains(&Keycode::Right) {
            if !right_held {
                right_held = true;
                send_command(&mut port, TURN_RIGHT_COMMAND, 15)?;
            }
        } else {
            right_held = false;
        }

        /*
         * Up arrow: 6 inch forward (once per press).
         */
        if keys.contains(&Keycode::Up) {
            if !up_held {
                up_held = true;
                send_command(&mut port, FORWARD_COMMAND, 6)?;
            }
        } else {
            up_held = false;
        }

        /*
         * Down arrow: 6 inch backward (once per press).
         */
        if keys.contains(&Keycode::Down) {
            if !down_held {
                down_held = true;
                send_command(&mut port, BACKWARD_COMMAND, 6)?;
            }
        } else {
            down_held = false;
        }

        macro_rules! once_key {
            ($key:expr, $held:expr, $cmd:expr, $amt:expr) => {
                if keys.contains(&$key) {
                    if !$held { $held = true; send_command(&mut port, $cmd, $amt)?; }
                } else { $held = false; }
            }
        }
        once_key!(Keycode::X, x_held, FORWARD_COMMAND,  1);
        once_key!(Keycode::C, c_held, FORWARD_COMMAND,  2);
        once_key!(Keycode::V, v_held, FORWARD_COMMAND,  4);
        once_key!(Keycode::B, b_held, FORWARD_COMMAND,  6);
        once_key!(Keycode::N, n_held, FORWARD_COMMAND, 10);
        once_key!(Keycode::M, m_held, FORWARD_COMMAND, 14);

        thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
    }

    terminal::disable_raw_mode()?;
    println!("Exiting controller.");
    Ok(())
}
