//! Initialization code

#![no_std]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust53964
extern crate panic_itm; // panic handler

pub use cortex_m::{asm::bkpt, iprint, iprintln, peripheral::ITM};
pub use cortex_m_rt::entry;
pub use f3::hal::{prelude, serial::Serial, stm32f30x::usart1, time::MonoTimer};

use core::fmt;

use f3::hal::{
    prelude::*,
    stm32f30x::{self, gpioc, GPIOE, RCC, USART1},
};

#[macro_export]
macro_rules! uprint {
    ($serial:expr, $($arg:tt)*) => {
        $serial.write_fmt(format_args!($($arg)*)).ok()
    };
}

#[macro_export]
macro_rules! uprintln {
    ($serial:expr, $fmt:expr) => {
        uprint!($serial, concat!($fmt, "\n"))
    };
    ($serial:expr, $fmt:expr, $($arg:tt)*) => {
        uprint!($serial, concat!($fmt, "\n"), $($arg)*)
    };
}

pub struct SerialPort {
    usart1: &'static mut usart1::RegisterBlock,
}

impl SerialPort {
    pub fn wait_for_write(&mut self) {
        while self.usart1.isr.read().txe().bit_is_clear() {}
    }
    pub fn write_byte(&mut self, byte: u8) {
        self.wait_for_write();
        self.usart1.tdr.write(|w| w.tdr().bits(u16::from(byte)))
    }
    pub fn wait_for_read(&mut self) {
        while self.usart1.isr.read().rxne().bit_is_clear() {}
    }
    pub fn read_byte(&mut self) -> u8 {
        self.wait_for_read();
        self.usart1.rdr.read().rdr().bits() as u8
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}


pub fn init_serial() -> (SerialPort, MonoTimer, ITM) {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb);

    let tx = gpioc.pc4.into_af7(&mut gpioc.moder, &mut gpioc.afrl);
    let rx = gpioc.pc5.into_af7(&mut gpioc.moder, &mut gpioc.afrl);

    Serial::usart1(dp.USART1, (tx, rx), 115_200.bps(), clocks, &mut rcc.apb2);

    unsafe {
        (
            SerialPort {
                usart1: &mut *(USART1::ptr() as *mut _),
            },
            MonoTimer::new(cp.DWT, clocks),
            cp.ITM,
        )
    }
}

pub fn init_onboard_leds() -> &'static gpioc::RegisterBlock {
    // restrict access to the other peripherals
    (stm32f30x::Peripherals::take().unwrap());

    let (gpioe, rcc) = unsafe { (&*GPIOE::ptr(), &*RCC::ptr()) };
    rcc.ahbenr.write(|w| w.iopeen().set_bit());

    gpioe.moder.write(|w| {
        w.moder8().output();
        w.moder9().output();
        w.moder10().output();
        w.moder11().output();
        w.moder12().output();
        w.moder13().output();
        w.moder14().output();
        w.moder15().output()
    });

    gpioe
}
