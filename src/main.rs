//#![deny(unsafe_code)]
//#![deny(warnings)]
#![no_main]
#![no_std]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust53964
extern crate panic_semihosting; // panic handler

#[allow(unused)]
use cortex_m_semihosting::hprintln;

use core::{
    fmt::Write,
    iter::{once, successors},
};

use embedded_hal::digital::v2::OutputPin;
use rtfm::{app, Instant};
use stm32f1xx_hal::{
    afio::AfioExt,
    flash::FlashExt,
    gpio::{
        gpioa::{PA5, PA6, PA7},
        gpiob::{PB10, PB11, PB12, PB13, PB14, PB15},
        gpioc::PC13,
        Alternate, Floating, GpioExt, Input, OpenDrain, Output, PullDown, PushPull,
    },
    i2c::{BlockingI2c, DutyCycle, Mode},
    pwm::PwmExt,
    rcc::RccExt,
    spi::Spi,
    stm32::{I2C2, SPI1},
    time::U32Ext,
};

use apa102_spi::Apa102;
#[allow(unused)]
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, RGB8,
};

use embedded_graphics::{fonts::Font6x8, prelude::*};
use ssd1306::{interface::I2cInterface, prelude::*, Builder};

use heapless::{consts, String, Vec};

use glow::knob::{Direction, Knob};
use glow::pwmled::PwmLed;

const PERIOD: u32 = 80_000;

#[app(device = stm32f1::stm32f103)]
const APP: () = {
    static mut toggle: bool = false;
    static mut led: PC13<Output<PushPull>> = ();
    //static mut button: PB12<Input<PullDown>> = ();
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
    static mut pwm_led: PwmLed = ();
    static mut speed: f32 = -0.65;
    static mut step: f32 = 1.0;
    static mut hue: f32 = 0.0;

    #[init(schedule = [tick])]
    fn init() -> init::LateResources {
        let rcc = device.RCC;
        let afio = device.AFIO;
        let exti = device.EXTI;
        let tim4 = device.TIM4;

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
            .sysclk(16.mhz())
            .freeze(&mut flash.acr);
        let mut afio = afio.constrain(&mut rcc.apb2);
        let mut gpioa = device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
        let mut gpioc = device.GPIOC.split(&mut rcc.apb2);

        let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
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
            //ws2812_spi::MODE,
            1_000_000.hz(),
            clocks,
            &mut rcc.apb2,
        );
        let led_strip = Apa102::new(spi);
        //let led_strip = Ws2812::new(spi);

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

        let c1 = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
        let c2 = gpiob.pb7.into_alternate_push_pull(&mut gpiob.crl);
        let c3 = gpiob.pb8.into_alternate_push_pull(&mut gpiob.crh);
        let c4 = gpiob.pb9.into_alternate_push_pull(&mut gpiob.crh);

        let (r, g, _, b) = tim4.pwm(
            (c1, c2, c3, c4),
            &mut afio.mapr,
            1.khz(),
            clocks,
            &mut rcc.apb1,
        );

        let pwm_led = PwmLed::new(r, g, b);

        schedule.tick(Instant::now() + PERIOD.cycles()).unwrap();
        init::LateResources {
            pwm_led,
            led,
            //button,
            knob,
            knob2,
            led_strip,
            screen,
        }
    }

    //#[interrupt(resources = [led, button, knob, speed, led_strip])]
    #[interrupt(resources = [led, knob, knob2, step, speed])]
    fn EXTI15_10() {
        use Direction::*;
        match resources.knob2.poll() {
            Some(CW) => {
                //let _ = resources.led_strip.write(all_red.iter().cloned());
                let _ = resources.led.set_high();
                *resources.step += 0.25;
            }
            Some(CCW) => {
                //let _ = resources.led_strip.write(all_blue.iter().cloned());
                let _ = resources.led.set_low();
                *resources.step -= 0.25;
                //cortex_m::asm::bkpt();
                //hprintln!("step: {}, speed: {}", *resources.step, *resources.speed);
            }
            None => {}
        }
        match resources.knob.poll() {
            Some(CW) => {
                //let _ = resources.led_strip.write(all_red.iter().cloned());
                let _ = resources.led.set_high();
                *resources.speed += 0.05;
            }
            Some(CCW) => {
                //let _ = resources.led_strip.write(all_blue.iter().cloned());
                let _ = resources.led.set_low();
                *resources.speed -= 0.05;
            }
            None => {}
        }
    }

    #[task(resources = [led_strip, hue, step, speed, screen], schedule = [tick])]
    fn tick() {
        let mut step_s: String<consts::U16> = String::new();
        let mut speed_s: String<consts::U16> = String::new();
        let mut hue_s: String<consts::U16> = String::new();
        let _ = write!(step_s, "step: {}", *resources.step);
        let _ = write!(speed_s, "speed: {}", *resources.speed * 100.0);
        let _ = write!(hue_s, "hue: {}", *resources.hue);
        let _ = resources.screen.clear();
        resources.screen.draw(
            Font6x8::render_str(step_s.as_str())
                .with_stroke(Some(1u8.into()))
                .into_iter(),
        );
        resources.screen.draw(
            Font6x8::render_str(speed_s.as_str())
                .with_stroke(Some(1u8.into()))
                .translate(Coord::new(0, 8))
                .into_iter(),
        );
        resources.screen.draw(
            Font6x8::render_str(hue_s.as_str())
                .with_stroke(Some(1u8.into()))
                .translate(Coord::new(0, 16))
                .into_iter(),
        );
        let _ = resources.screen.flush();
        schedule.tick(Instant::now() + PERIOD.cycles()).unwrap();
        let hue = (((*resources.hue + *resources.speed) % 192.0) + 192.0) % 192.0;
        *resources.hue = hue;
        let start = Hsv {
            hue: hue as u8,
            sat: 0x99,
            val: 0x55,
        };
        let inc = *resources.step as u8;
        let colors = successors(Some(start), |c| {
            Some(Hsv {
                hue: ((c.hue as u16 + inc as u16) % 192) as u8,
                ..*c
            })
        })
        .map(hsv2rgb)
        .take(72);
        let block: Vec<RGB8, consts::U72> = colors.collect();
        let center = once(block[0]).cycle().take(2);
        let petals = once(block[1]).chain(once(block[2])).cycle().take(12);
        let rays = once(block[3]).chain(once(block[4])).cycle().take(12);
        let outer = once(block[5]).chain(once(block[6])).cycle().take(12);
        let full = center.chain(petals).chain(rays).chain(outer);
        //let full_block: Vec<RGB8, consts::U38> = full.collect();
        let _ = resources.led_strip.write(full);
    }

    extern "C" {
        fn EXTI0();
    }
};
