extern crate libc;
extern crate signal_hook;

mod recorder;
mod serial;
mod utils;
// mod backoff;
use clap::Parser;
use env_logger::{Builder, Env};
use log::*;
use serialport::{ClearBuffer, SerialPort};
use std::borrow::{Borrow, BorrowMut};
use std::io::{ErrorKind, Write};
use std::time::Duration;
use utils::ResultError;

use crate::utils::SignalState;
use crate::recorder::Recorder;

const SYSTEM: &str = "sky360";
const SUB_SYSTEM: &str = "cyclop";

// ros parameters
const SENSOR_NAME: &str = "gps";
const SERIAL_PORT: &str = "/dev/ttyACM0";
const BAUDE_RATE: u32 = 9600;
const BUFFER_SIZE: usize = 4096; // bytes - should match partition block size at minimum
const TIMEOUT: u64 = 1000; // ms

//TODO: ros2
//      we need to ensure the message timestamp = ros timestamp
//TODO: how to deal incoming messages ??? -
//      start with #2 - file per read and continue with #3 to improve performance and diskspace usage

// 1. file + timestamp per message
//    if we write each buffer to its own file we get time resolve but increase disk usage at the cost of simplicity
//    this is not as simple as we can receive multiple messages per read
// *2. file + timestamp per read
//    this is aligned with the time requirements as the multiple messages were received at the same time.
//    [8 bytes timestamp] [2 bytes buffer length] [buffer]
//
// 3. batching of messages per file
//    can be its own ros2 node that will batch "old" events to a single file for long term storage!
//    file is append only
//    size can match partition block size (4k usually)
//    might be worth to hae the file per frequencey (1s, 5s, etc)
//    requires protocol and more complex approach to store timestamp with the raw data
//    protocol as prefix and suffixx to sepeate between messages
//    we lose this way the indexing of files as now we will need to scan a file for a specific message
//    will be fine for playback
//    publishing a message requires now the name of the file it was stored at and the position + size

#[derive(Parser, Debug, Clone, clap::ValueEnum, PartialEq)]
#[clap(rename_all = "kebab_case")]
enum OutputFormat {
    Hex,
    Text,
}

#[derive(Parser)]
#[command()]
struct Cli {
    /// Lists available serial devices and exits
    #[arg(short, long)]
    device_list: bool,

    /// Sets the output format (default: hex; options: hex, text)
    #[arg(short, long, default_value = "text")]
    output: OutputFormat,
}

fn main() {
    let cli = Cli::parse();
    if cli.device_list {
        serial::print_serial_devices();
        return ();
    }

    Builder::from_env(Env::default().default_filter_or("trace")).init();
    match start(SERIAL_PORT, BAUDE_RATE, BUFFER_SIZE, TIMEOUT, &cli.output) {
        Err(e) => {
            error!("Panic: {:?}", e);
            panic!("Terminated");
        }
        Ok(_) => {
            info!("Goodbye...");
            return;
        }
    }
}

fn start(
    serial_port: &str,
    baud_rate: u32,
    buffer_size: usize,
    timeout: u64,
    output_format: &OutputFormat,
) -> ResultError<()> {
    info!("Initializing");

    let path = format! ("/tmp/{}/data/{}/sensors/{}",SYSTEM, SUB_SYSTEM, SENSOR_NAME);
    let mut recorder = Recorder::new(path)?;
    
    let mut signals = utils::setup_signal_handler()?;
    let mut port = initialize_serial_port(serial_port, baud_rate, timeout)?;

    //TODO: reset device - recycle the power of the serial_port - write a function
    info!("Starting");
    loop {
        match utils::check_signals(&mut signals) {
            SignalState::None => {}
            SignalState::ReloadConfig => {}
            SignalState::Exit => return Ok(()),
        }
        
        let bytes_read =
            process_serial_data(&mut port, &mut recorder, buffer_size, output_format)?.unwrap_or_default();

        
        debug!("Read {} bytes from {}", bytes_read, serial_port);
    }
}

fn initialize_serial_port(
    serial_port: &str,
    baud_rate: u32,
    timeout: u64,
) -> ResultError<Box<dyn SerialPort>> {
    let port = serialport::new(serial_port, baud_rate)
        .timeout(Duration::from_millis(timeout))
        .open()?;

    port.clear(ClearBuffer::All)?;
    info!("Serial port initialized");
    Ok(port)
}

fn process_serial_data(
    port: &mut Box<dyn SerialPort>,
    recorder: &mut Recorder,
    buffer_size: usize,
    output_format: &OutputFormat
) -> ResultError<Option<usize>> {

    //TODO: ROS2:SYSTEM_TIME recorder create buffer needs to be captured 
    let buffer = recorder.create_buffer(buffer_size)?;

    //TODO: start sensor read time
    match port.read(buffer) { 
        Err(e) => match e.kind() {
            ErrorKind::TimedOut => {
                warn!("Warning: {}", e.to_string());
                //TODO: wait with PID backoff
                return Ok(None);
            }
            ErrorKind::Interrupted => {
                warn!("Warning: {}", e.to_string());
                return Ok(None);
            }
            _ => {
                error!("Error: Unknown: {:?}", e);
                return Err(e.into());
            }
        },
        Ok(bytes_read) => {
            //TODO: required? (i think this is requires once the recorder uses DmaBuffer)
            buffer.as_mut().flush()?; //TODO: how we can capture this time? (system time) and dump it with the suffix for the protocol? maybe update the file's modified date?
            //TODO: measure sensor read time

            //TODO: start processing timer
            let data = &buffer[..bytes_read];
            match *output_format {
                OutputFormat::Hex => {
                    trace!("{:x?}", data);
                }
                OutputFormat::Text => {
                    trace!("{}", String::from_utf8_lossy(data).trim());
                }
            }
            // TODO: measure processing time
            return Ok(Some(bytes_read));
        }
    }
}
