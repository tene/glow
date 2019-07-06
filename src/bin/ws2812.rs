#![no_main]
#![no_std]

#[allow(unused)]
extern crate panic_itm; // panic handler

use stm32f1xx_hal as hal;
use ws2812_timer_delay as ws2812;

use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::stm32;
use crate::hal::time::*;
use crate::hal::timer::*;
use crate::ws2812::Ws2812;
use cortex_m::asm;
use cortex_m::peripheral::Peripherals;

use smart_leds::{hsv::RGB, SmartLedsWrite, RGB8};

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        // Constrain clocking registers
        let mut flash = p.FLASH.constrain();
        let mut rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.freeze(&mut flash.acr);
        let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = p.GPIOB.split(&mut rcc.apb2);

        /* (Re-)configure PA7 as output */
        let mut ws_data_pin = gpioa.pa7.into_push_pull_output(&mut gpioa.crl);
        let mut test = gpiob.pb5.into_push_pull_output(&mut gpiob.crl);
        /*
        ws_data_pin.set_low();
        asm::bkpt();
        ws_data_pin.set_high();
        asm::bkpt();
        ws_data_pin.set_low();
        asm::bkpt();
        */

        let timer = Timer::tim1(p.TIM1, MegaHertz(3), clocks.clone(), &mut rcc.apb2);

        // Get delay provider
        let mut delay = Delay::new(cp.SYST, clocks.clone());

        let mut ws = Ws2812::new(timer, &mut ws_data_pin);
        let mut data: [RGB8; 3] = [RGB::default(); 3];
        let empty: [RGB8; 3] = [RGB::default(); 3];

        data[0] = RGB::new(0, 0, 0x10);
        data[1] = RGB::new(0, 0x10, 0);
        data[2] = RGB::new(0x10, 0, 0);

        loop {
            ws.write(data.iter().cloned()).unwrap();
            delay.delay_ms(10 as u16);
            ws.write(empty.iter().cloned()).unwrap();
            delay.delay_ms(10 as u16);
        }
    }
    loop {
        continue;
    }
}
