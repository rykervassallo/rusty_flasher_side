use serialport::SerialPort;
use crate::tty_echo_serialport::echo;
use crate::{generate_crc::crc, read_file};
use std::time::Duration;
use std::{fs::read, io, thread, time};
use crate::find_tty_serialport::find_tty;
use crate::read_file::read_file;
use crate::parse_invocation::parse_invocation;


/*
 * Relevant constants.
 */
const BOOT_ACK       : u8 = 0x21;
const BOOT_START     : u8 = 0x67;
const GET_PROG_INFO  : u8 = 0x68;
const PUT_PROG_INFO  : u8 = 0x69;
const GET_CODE       : u8 = 0x70;
const PUT_CODE       : u8 = 0x71;
const BOOT_SUCCESS   : u8 = 0x72;
const BOOT_ERROR_BIG : u8 = 0x73;
const BOOT_ERROR_CRC : u8 = 0x74;

pub fn full_boot() -> io::Result<()> {
    /*
     * Grab filepath.
     */
    let filepath : std::path::PathBuf = parse_invocation()?;

    /*
     * Grab port.
     */
    let mut port = find_tty()?;

    /*
     * Grab file to send.
     */
    let file : Vec<u8> = read_file(&filepath)?;

    /*
     * Boot to Pi.
     */
    const BOOT_CODE_ADDRESS : u32 = 0x8000;
    send_boot(&mut port, file, BOOT_CODE_ADDRESS)?;

    /*
     * Echo serial port.
     */
    echo(&mut port)?;

    /*
     * Return Ok if nominal, error prop happens earlier with '?' (enormous Rust W).
     */
    Ok(())
}

pub fn test_boot() -> io::Result<()> {
    /*
     * Grab port.s
     */
    let mut port = find_tty()?;

    let mut my_file: Vec<u8> = vec![]; 
    for i in 0..100000 { my_file.push((i % 255) as u8); }
    // let my_file = vec![1,2,3,4,5,6,7,8];

    const BOOT_CODE_ADDRESS : u32 = 0x8000;
    send_boot(&mut port, my_file, BOOT_CODE_ADDRESS)?;

    Ok(())
}

pub fn send_boot(port : &mut Box<dyn SerialPort>, file : Vec<u8>, address : u32) -> io::Result<()> {
    let ERROR_NO_START_CODE = io::Error::new(
        io::ErrorKind::NotFound,
        "Not seeing code request--hit reset?",
    );

    let ERROR_UNEXPECTED_BYTE_COUNT = io::Error::new(
        io::ErrorKind::NotFound,
        "Unexpected byte count",
    );

    let ERROR_UNEXPECTED_NETWORK_CODE = io::Error::new(
        io::ErrorKind::NotFound,
        "Unexpected network code",
    );

    let ERROR_TOO_BIG = io::Error::new(
        io::ErrorKind::FileTooLarge,
        "Compiled binary too big for pi",
    );

    let ERROR_DOWNLOAD_ISSUE = io::Error::new(
        io::ErrorKind::NetworkDown,
        "Compiled binary too big for pi",
    );

    let ERROR_DOWNLOAD_ISSUE_CRC = io::Error::new(
        io::ErrorKind::NetworkDown,
        "CRC mismatch",
    );

    /*
     * First get crc.
     */
    let our_crc : u32 = crc(&file);
    let mut buf: [u8; 1024] = [0u8 ; 1024];

    /*
     * Wait until we see GET_PROG_INFO, this should be immediate but still.
     */
    let n : usize = read_bytes(port, &mut buf)?;
    let byte = buf[0];
    if (byte != GET_PROG_INFO) { return Err(ERROR_NO_START_CODE); } 
    if (n != 1) { return Err(ERROR_UNEXPECTED_BYTE_COUNT); }

    /*
     * Construct program info packet.
     */
    let mut prog_info_bytes : [u8 ; 13] = [0u8 ; 13];
    prog_info_bytes[0] = PUT_PROG_INFO;
    prog_info_bytes[1..5].copy_from_slice(&address.to_be_bytes());
    prog_info_bytes[5..9].copy_from_slice(&our_crc.to_be_bytes());
    prog_info_bytes[9..13].copy_from_slice(&(file.len() as u32).to_be_bytes());

    /*
     * Send packet.
     */
    send_wait_ack(port, &prog_info_bytes)?;

    /*
     * Next, we expect a GET_CODE.
     */
    let n : usize = read_bytes(port, &mut buf)?;
    let code : u8 = buf[0];
    if (code == BOOT_ERROR_BIG) { return Err(ERROR_TOO_BIG); }
    if (n != 1) { return Err(ERROR_UNEXPECTED_BYTE_COUNT); }
    else if (code != GET_CODE) { return Err(ERROR_UNEXPECTED_NETWORK_CODE); }

    /*
     * Now, we send the program.
     * The below code is TOO SLOW!
     */
    // send_wait_ack(&mut port, &file)?;    
    print!("Sending code now, size is {} and crc is 0x{:x}\n", file.len(), our_crc);
    let delay = time::Duration::from_micros(10);
    for bytes in file.chunks(8) {
        port.write_all(bytes)?;
        port.flush()?;

        /*
         * Wait for pi to process.
         */
        thread::sleep(delay);
    }

    /*
     * Read back status.
     */
    let mut buf_one_byte : [u8 ; 1] = [0u8 ; 1];
    let n : usize = read_bytes(port, &mut buf_one_byte)?;
    if (n != 1) { return Err(ERROR_UNEXPECTED_BYTE_COUNT); }
    let code : u8 = buf_one_byte[0];
    if (code == BOOT_ERROR_CRC) { return Err(ERROR_DOWNLOAD_ISSUE_CRC); }
    else if (code != BOOT_SUCCESS) { return Err(ERROR_UNEXPECTED_NETWORK_CODE); }

    print!("Boot success!\n");

    /*
     * Boot was a success.
     */
    Ok(())
}

fn read_bytes(port : &mut Box<dyn SerialPort>, buf : &mut [u8]) -> io::Result<usize> {
    loop {
        match port.read(buf) {
            /*
             * Reading bytes.
             */
            Ok(n) if n > 0 => {
                return Ok(n);
            }

            /*
             * Reading nothing (successfully).
             */
            Ok(_) => {
                return Ok(0);
            }

            /*
             * Timeout.
             */
            Err(e) if e.kind() == io::ErrorKind::TimedOut => {
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

fn send_wait_ack(port : &mut Box<dyn SerialPort>, buf : &[u8]) -> io::Result<()> {
    let ERROR_DOWNLOAD_ISSUE = io::Error::new(
        io::ErrorKind::NetworkDown,
        "Compiled binary too big for pi",
    );

    const RPI_UART_RX_BUFFER_SIZE : usize = 8;
    const HOST_BUF_SIZE : usize = 1024;
    let mut response_buffer : [u8 ; HOST_BUF_SIZE] = [0u8 ; HOST_BUF_SIZE];
    let NUM_CHUNKS : usize = (buf.len() + RPI_UART_RX_BUFFER_SIZE - 1) / RPI_UART_RX_BUFFER_SIZE;
    for (which_chunk, chunk ) in buf.chunks(RPI_UART_RX_BUFFER_SIZE).into_iter().enumerate() {
        // print!("writing chunk {}\n", which_chunk);
        port.write_all(chunk)?;
        port.flush()?;
        if (which_chunk < NUM_CHUNKS - 1) {
            // print!("getting ack for chunk {}\n", which_chunk);
            let n : usize = read_bytes(port, &mut response_buffer)?;
            if (n != 1 || response_buffer[0] != BOOT_ACK) { 
                print!("ack is actually '{:x}' here, n is {}\n", response_buffer[0], n);
                return Err(ERROR_DOWNLOAD_ISSUE) 
            }
            // print!("got ack from chunk {}\n", which_chunk);
        }
    }

    Ok(())
}
