#![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::fmt::Write;

#[allow(unused_imports)]
use glow::{entry, iprint, iprintln, uprint, uprintln, usart1, StSerial, System};

use heapless::{consts, Vec};

#[entry]
fn main() -> ! {
    let system = System::new();
    let mut serial = system.init_serial();

    uprintln!(serial, "The answer is {}", 40 + 2);

    //let mut buffer: Vec<u8, consts::U32> = Vec::new();

    loop {
        let byte = serial.read();
        serial.write(byte);
        //aux11::bkpt();
        //uprint!(serial, "{}", byte as char);
        /*if byte == 0x0a || byte == 0x0d {
            buffer.clear();
        } else {
            let _ = buffer.push(byte);
        }*/
    }
}
