use serialport::SerialPort;
use crate::{generate_crc::crc, read_file};
use std::{fs::read, io};
use crate::find_tty_serialport::find_tty;
use crate::read_file::read_file;
use crate::parse_invocation::parse_invocation;


/*
 * Relevant constants.
 */
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
    let port = find_tty()?;

    /*
     * Grab file to send.
     */
    let file : Vec<u8> = read_file(&filepath)?;

    /*
     * Boot to Pi.
     */
    send_boot(port, file, 10)?;

    /*
     * Return Ok if nominal, error prop happens earlier with '?' (enormous Rust W).
     */
    Ok(())
}

pub fn test_boot() -> io::Result<()> {
    /*
     * Grab port.s
     */
    let port = find_tty()?;
    let my_file = vec![1,2,3,4]; 
    send_boot(port, my_file, 0)?;

    Ok(())
}

pub fn send_boot(mut port : Box<dyn SerialPort>, file : Vec<u8>, address : u32) -> io::Result<()> {
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

    /*
     * First get crc.
     */
    let crc : u32 = crc(&file);
    let mut buf: [u8; 1024] = [0u8 ; 1024];

    /*
     * Wait until we see GET_PROG_INFO, this should be immediate but still.
     */
    loop {
        let n : usize = read_bytes(&mut port, &mut buf)?;
        if (n != 1) { return Err(ERROR_UNEXPECTED_BYTE_COUNT); }
        let byte = buf[0];
        if (byte == GET_PROG_INFO) { break; } 
        else { return Err(ERROR_UNEXPECTED_NETWORK_CODE); }
    }

    print!("Received GET_PROG_INFO, sending PUT_PROG_INFO now\n");

    /*
     * Construct program info packet.
     */
    let mut prog_info_bytes : [u8 ; 13] = [0u8 ; 13];
    prog_info_bytes[0] = PUT_PROG_INFO;
    prog_info_bytes[1..5].copy_from_slice(&address.to_be_bytes());
    prog_info_bytes[5..9].copy_from_slice(&crc.to_be_bytes());
    prog_info_bytes[9..13].copy_from_slice(&(file.len() as u32).to_be_bytes());
    // prog_info_bytes[1] = 0x13;

    /*
     * Send packet.
     */
    port.write_all(&prog_info_bytes)?;
    port.flush()?;

    /*
     * Next, we expect a GET_CODE.
     */
    loop {
        let n : usize = read_bytes(&mut port, &mut buf)?;
        if (n != 1) { return Err(ERROR_UNEXPECTED_BYTE_COUNT); }
        let code : u8 = buf[0];
        if (code == GET_PROG_INFO) { continue; } // Flush extra GET_PROG_INFOs
        else if (code == GET_CODE) { break; }
        else if (code == BOOT_ERROR_BIG) { return Err(ERROR_TOO_BIG); }
        else { return Err(ERROR_UNEXPECTED_NETWORK_CODE); }
    }

    print!("Received a 'GET_CODE'\n");

    return Ok(());

    /*
     * Now, we send the program.
     */
    let put_code : [u8; 1] = [PUT_CODE];
    port.write_all(&put_code)?;
    port.flush()?;

    const CHUNK_SIZE : usize = 256;
    for chunk in file.chunks(CHUNK_SIZE) {
        port.write_all(chunk)?;
        port.flush()?;
    }

    /*
     * Read back status.
     */
    let n : usize = read_bytes(&mut port, &mut buf)?;
    if (n != 1) { return Err(ERROR_UNEXPECTED_BYTE_COUNT); }
    let code : u8 = buf[0];
    if (code == BOOT_ERROR_CRC) { return Err(ERROR_DOWNLOAD_ISSUE); }
    else if (code != BOOT_SUCCESS) { return Err(ERROR_UNEXPECTED_NETWORK_CODE); }

    /*
     * Boot was a success.
     */




    Ok(())
}

fn read_bytes(port : &mut Box<dyn SerialPort>, buf : &mut [u8 ; 1024]) -> io::Result<usize> {
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