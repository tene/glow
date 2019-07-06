#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use apa102_spi as apa102;
use stm32f1xx_hal as hal;

use crate::apa102::Apa102;
use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::spi::Spi;
use crate::hal::stm32;
use cortex_m::Peripherals;

use smart_leds::{hsv::RGB, SmartLedsWrite, RGB8};

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (hal::pac::Peripherals::take(), Peripherals::take()) {
        // Constrain clocking registers
        let mut flash = p.FLASH.constrain();
        let mut rcc = p.RCC.constrain();
        let mut afio = p.AFIO.constrain(&mut rcc.apb2);
        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(16.mhz())
            .freeze(&mut flash.acr);
        let mut gpioa = p.GPIOA.split(&mut rcc.apb2);

        // Get delay provider
        let mut delay = Delay::new(cp.SYST, clocks);

        // Configure pins for SPI
        let (sck, miso, mosi) = cortex_m::interrupt::free(move |cs| {
            (
                gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl),
                gpioa.pa6.into_floating_input(&mut gpioa.crl),
                gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl),
            )
        });

        // Configure SPI with 3Mhz rate
        let spi = Spi::spi1(
            p.SPI1,
            (sck, miso, mosi),
            &mut afio.mapr,
            apa102::MODE,
            1_000_000.hz(),
            clocks,
            &mut rcc.apb2,
        );
        const MAX: usize = 8;
        const COLOR1: RGB8 = RGB::new(0xff, 0x05, 0x00);
        const COLOR2: RGB8 = RGB::new(0x00, 0x24, 0xff);
        let mut data: [RGB8; MAX] = [(0, 0, 0).into(); MAX];
        let mut main = 0;
        let mut apa = Apa102::new(spi);
        let mut up = true;
        loop {
            for i in 0..MAX {
                let distance = (main as i32 - i as i32).abs() as u8;
                let c1 = (
                    COLOR1.r as u32 * (MAX as u32 - distance as u32) / MAX as u32,
                    COLOR1.g as u32 * (MAX as u32 - distance as u32) / MAX as u32,
                    COLOR1.b as u32 * (MAX as u32 - distance as u32) / MAX as u32,
                );
                let c2 = (
                    COLOR2.r as u32 * distance as u32 / MAX as u32,
                    COLOR2.g as u32 * distance as u32 / MAX as u32,
                    COLOR2.b as u32 * distance as u32 / MAX as u32,
                );
                let ct = (
                    (c1.0 + c2.0) as u8,
                    (c1.1 + c2.1) as u8,
                    (c1.2 + c2.2) as u8,
                )
                    .into();
                data[i] = ct;
            }
            if up {
                if main == MAX - 1 {
                    up = false;
                    main -= 2;
                }
                main += 1;
            } else {
                if main == 0 {
                    up = true;
                    main += 2;
                }
                main -= 1;
            }
            apa.write(data.iter().cloned()).unwrap();
            delay.delay_ms(100 as u16);
        }
    }
    loop {
        continue;
    }
}
