#![no_std]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust53964
extern crate panic_itm; // panic handler

pub use cortex_m::{asm::bkpt, iprint, iprintln, peripheral::ITM};
pub use cortex_m_rt::entry;
pub use f3::hal::{prelude, stm32f30x::usart1, stm32f30x::TIM2, time::MonoTimer};

use core::fmt;

use f3::hal::{
    gpio::{
        gpioa::{self, PAx},
        Output, PushPull,
    },
    prelude::*,
    serial::{Rx, Serial, Tx},
    stm32f30x::{self, gpioc, GPIOE, RCC, USART1},
};
/*
use f3::hal::{
    gpio::{
        gpioa::{self, PAx},
        Output, PushPull,
    },
    prelude::*,
    serial::{Rx, Serial, Tx},
    stm32f30x::{self, gpioc, GPIOE, RCC, USART1},
};
*/

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

    pub fn init_pwm(self) -> TIM2 {
        let mut rcc = self.dp.RCC;
        let mut gpioa = self.dp.GPIOA;
        let mut tim2 = self.dp.TIM2;

        // Turn on GPIOA
        rcc.ahbenr.modify(|_r, w| w.iopaen().set_bit());

        // Set pins to AF1 (TIM2_CH_2-4)
        gpioa.moder.modify(|_r, w| {
            w.moder1().alternate();
            w.moder2().alternate();
            w.moder3().alternate()
        });

        gpioa.afrl.modify(|_r, w| unsafe {
            w.afrl1().bits(1);
            w.afrl2().bits(1);
            w.afrl3().bits(1)
        });

        // Power on TIM2
        rcc.apb1enr.modify(|_r, w| w.tim2en().set_bit());

        // Configure TIM2 channels 2-4 for PWM mode 1
        tim2.ccmr1_output
            .modify(|_r, w| unsafe { w.oc2m().bits(0b110).oc2pe().set_bit() });
        tim2.ccmr2_output.modify(|_r, w| unsafe {
            w.oc3m().bits(0b110).oc4m().bits(0b110);
            w.oc3pe().set_bit().oc4pe().set_bit()
        });
        tim2.ccer.modify(|_r, w| {
            w.cc2e().set_bit();
            w.cc3e().set_bit();
            w.cc4e().set_bit()
        });
        //tim2.cr1.modify(|_r, w| w.arpe().set_bit());

        // Configure TIM2 to run at 1kHz
        tim2.arr.write(|w| unsafe { w.arrl().bits(256) });
        tim2.psc.write(|w| unsafe { w.psc().bits(7999) });

        // Set starting color
        tim2.ccr2.write(|w| unsafe { w.bits(178) });
        tim2.ccr3.write(|w| unsafe { w.bits(102) });
        tim2.ccr4.write(|w| unsafe { w.bits(255) });

        // Start timer
        tim2.cr1.modify(|_r, w| w.cen().set_bit());
        tim2.egr.write(|w| w.ug().set_bit());

        tim2
    }
}
