#![no_std]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust53964
extern crate panic_itm; // panic handler

pub use cortex_m::{asm::bkpt, iprint, iprintln, peripheral::ITM};
pub use cortex_m_rt::entry;
pub use f3::hal::{prelude, stm32f30x::usart1, time::MonoTimer};

use core::fmt;

use f3::hal::{
    serial::{Serial, Tx, Rx},
    gpio::{
        gpioa::{self, PAx},
        Output, PushPull,
    },
    prelude::*,
    stm32f30x::{self, gpioc, GPIOE, RCC, USART1},
};

use nb::block;

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

pub struct StSerial {
    tx: Tx<USART1>,
    rx: Rx<USART1>,
}

impl fmt::Write for StSerial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            let _ = self.tx.write(byte);
        }
        Ok(())
    }
}

impl StSerial {
    pub fn write(&mut self, byte: u8) {
        self.tx.write(byte).unwrap();
    }
    pub fn read(&mut self) -> u8 {
        block!(self.rx.read()).unwrap()
    }
}

pub struct System {
    cp: cortex_m::Peripherals,
    dp: stm32f30x::Peripherals,
}

impl System {
    pub fn new() -> Self {
        let cp = cortex_m::Peripherals::take().unwrap();
        let dp = stm32f30x::Peripherals::take().unwrap();
        Self { cp, dp }
    }

    pub fn init_serial(self) -> StSerial {
        let mut flash = self.dp.FLASH.constrain();
        let mut rcc = self.dp.RCC.constrain();

        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        let mut gpioc = self.dp.GPIOC.split(&mut rcc.ahb);

        let tx_pin = gpioc.pc4.into_af7(&mut gpioc.moder, &mut gpioc.afrl);
        let rx_pin = gpioc.pc5.into_af7(&mut gpioc.moder, &mut gpioc.afrl);

        let (tx, rx) = Serial::usart1(
            self.dp.USART1,
            (tx_pin, rx_pin),
            115_200.bps(),
            clocks,
            &mut rcc.apb2,
        )
        .split();
        StSerial { tx, rx }
    }

    pub fn init_onboard_leds(self) -> stm32f30x::GPIOE {
        let rcc = self.dp.RCC;
        let gpioe = self.dp.GPIOE;

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

    pub fn init_pwm(
        self,
    ) -> (
        PAx<Output<PushPull>>,
        PAx<Output<PushPull>>,
        PAx<Output<PushPull>>,
    ) {
        let mut rcc = self.dp.RCC.constrain();
        let mut gpioa = self.dp.GPIOA.split(&mut rcc.ahb);

        let red = gpioa
            .pa0
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper)
            .downgrade();
        let green = gpioa
            .pa1
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper)
            .downgrade();
        let blue = gpioa
            .pa2
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper)
            .downgrade();

        (red, green, blue)
    }
}