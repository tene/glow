//#![deny(unsafe_code)]
//#![deny(warnings)]
#![no_main]
#![no_std]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust53964
extern crate panic_semihosting; // panic handler

#[allow(unused)]
use cortex_m_semihosting::hprintln;

//use embedded_hal::digital::v2::OutputPin;
use rtfm::{app, Instant};
use stm32f1xx_hal::{
    afio::AfioExt,
    flash::FlashExt,
    gpio::{
        gpioa::{PA5, PA6, PA7},
        gpiob::{PB10, PB11, PB12, PB13, PB14, PB15},
        Alternate, Floating, GpioExt, Input, OpenDrain, PullDown, PushPull,
    },
    i2c::{BlockingI2c, DutyCycle, Mode},
    rcc::RccExt,
    spi::Spi,
    stm32::{I2C2, SPI1},
    time::U32Ext,
};

use apa102_spi::Apa102;
#[allow(unused)]
use smart_leds::{SmartLedsWrite, RGB8};

use embedded_graphics::{fonts::Font6x8, prelude::*};
use ssd1306::{interface::I2cInterface, prelude::*, Builder};

use glow::knob::Knob;
use glow::m6::{Generator, Render};
use glow::render::{Breath, Rainbow};

const PERIOD: u32 = 800_000;
const DEBUG_PERIOD: u32 = 8_000_000;

#[app(device = stm32f1::stm32f103)]
const APP: () = {
    static mut knob: Knob<PB12<Input<PullDown>>, PB13<Input<PullDown>>> = ();
    static mut knob2: Knob<PB14<Input<PullDown>>, PB15<Input<PullDown>>> = ();
    static mut screen: GraphicsMode<
        I2cInterface<BlockingI2c<I2C2, (PB10<Alternate<OpenDrain>>, PB11<Alternate<OpenDrain>>)>>,
    > = ();
    static mut led_strip: Apa102<
        //static mut led_strip: Ws2812<
        Spi<
            SPI1,
            (
                PA5<Alternate<PushPull>>,
                PA6<Input<Floating>>,
                PA7<Alternate<PushPull>>,
            ),
        >,
    > = ();
    static mut rainbow: Rainbow = Rainbow::new();
    static mut breath: Breath = Breath::new();

    #[init(schedule = [tick, debug_tick])]
    fn init() -> init::LateResources {
        let rcc = device.RCC;
        let afio = device.AFIO;
        let exti = device.EXTI;

        rcc.apb2enr
            .modify(|_r, w| w.afioen().enabled().spi1en().enabled());
        afio.exticr4.modify(|_r, w| unsafe {
            w.exti12()
                .bits(0b001)
                .exti13()
                .bits(0b001)
                .exti14()
                .bits(0b001)
        });

        // Enable EXT Interrupts 12-14
        exti.imr
            .modify(|_r, w| w.mr12().set_bit().mr13().set_bit().mr14().set_bit());

        // Enable rising trigger for 12-14
        exti.rtsr
            .modify(|_r, w| w.tr12().set_bit().tr13().set_bit().tr14().set_bit());
        // Enable falling trigger for 12-14
        exti.ftsr
            .modify(|_r, w| w.tr12().set_bit().tr13().set_bit().tr14().set_bit());

        let mut rcc = rcc.constrain();
        let mut flash = device.FLASH.constrain();

        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(24.mhz())
            .freeze(&mut flash.acr);
        let mut afio = afio.constrain(&mut rcc.apb2);
        let mut gpioa = device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = device.GPIOB.split(&mut rcc.apb2);

        let k1a = gpiob.pb12.into_pull_down_input(&mut gpiob.crh);
        let k1b = gpiob.pb13.into_pull_down_input(&mut gpiob.crh);
        let k2a = gpiob.pb14.into_pull_down_input(&mut gpiob.crh);
        let k2b = gpiob.pb15.into_pull_down_input(&mut gpiob.crh);
        let knob = Knob::new(k1a, k1b);
        let knob2 = Knob::new(k2a, k2b);

        let pa5 = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
        let pa6 = gpioa.pa6.into_floating_input(&mut gpioa.crl);
        let pa7 = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

        let spi_pins = (pa5, pa6, pa7);
        let spi = Spi::spi1(
            device.SPI1,
            spi_pins,
            &mut afio.mapr,
            apa102_spi::MODE,
            24_000_000.hz(),
            clocks,
            &mut rcc.apb2,
        );
        let led_strip = Apa102::new(spi);

        let pb10 = gpiob.pb10.into_alternate_open_drain(&mut gpiob.crh);
        let pb11 = gpiob.pb11.into_alternate_open_drain(&mut gpiob.crh);
        let i2c_pins = (pb10, pb11);
        let i2c = BlockingI2c::i2c2(
            device.I2C2,
            i2c_pins,
            Mode::Fast {
                frequency: 400_000,
                duty_cycle: DutyCycle::Ratio2to1,
            },
            clocks,
            &mut rcc.apb1,
            1000,
            10,
            1000,
            1000,
        );
        let mut screen: GraphicsMode<_> = Builder::new()
            .with_size(DisplaySize::Display128x32)
            //.with_rotation(DisplayRotation::Rotate90)
            .connect_i2c(i2c)
            .into();
        screen.init().unwrap();
        screen.flush().unwrap();

        schedule.tick(Instant::now() + PERIOD.cycles()).unwrap();
        schedule
            .debug_tick(Instant::now() + DEBUG_PERIOD.cycles())
            .unwrap();
        init::LateResources {
            knob,
            knob2,
            led_strip,
            screen,
        }
    }

    #[interrupt(resources = [knob, knob2, rainbow, breath], priority=1)]
    fn EXTI15_10() {
        let k1 = &mut resources.knob;
        let k2 = &mut resources.knob2;
        resources.breath.lock(|r| {
        match k1.poll() {
            Some(x) => {
                r.knob1(x);
            }
            None => {}
        }
        match k2.poll() {
            Some(x) => {
                r.knob2(x);
            }
            None => {}
        }
        });
    }

    #[task(resources = [led_strip, rainbow, breath], schedule = [tick], priority = 2)]
    fn tick() {
        schedule.tick(Instant::now() + PERIOD.cycles()).unwrap();
        //let r = resources.breath as &mut dyn Render;
        let ls = resources.led_strip;
        resources.breath.lock(|r| {
        let g: Generator = Generator::new(r);
        let _ = ls.write(g);
        r.tick();
        });
    }

    #[task(resources = [screen, rainbow, breath], schedule = [debug_tick])]
    fn debug_tick() {
        schedule
            .debug_tick(Instant::now() + DEBUG_PERIOD.cycles())
            .unwrap();
        let dbgv = resources.breath.lock(|r| r.debug());
        let _ = resources.screen.clear();
        for i in 0..(dbgv.len()) {
        resources.screen.draw(
            Font6x8::render_str(dbgv[i].as_str())
                .with_stroke(Some(1u8.into()))
                .translate(Coord::new(0, 8*(i as i32)))
                .into_iter(),
        );
        }
        let _ = resources.screen.flush();
    }
    extern "C" {
        fn EXTI0();
        fn USART1();
    }
};
