#![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::fmt::Write;

#[allow(unused_imports)]
use glow::{entry, iprint, iprintln, uprint, uprintln, usart1, System};

use stm32f30x_hal::prelude::_embedded_hal_digital_OutputPin;

#[entry]
fn main() -> ! {
    let system = System::new();
    let mut tim2 = system.init_pwm();

    loop {
    }
}
