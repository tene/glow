//#![deny(unsafe_code)]
//#![deny(warnings)]
#![no_main]
#![no_std]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust53964
extern crate panic_itm; // panic handler

use cortex_m::iprintln;
use rtfm::{app, Instant};
use stm32f1::stm32f103::Interrupt;
use stm32f1xx_hal::{
    self as hal,
    gpio::{
        gpiob::{PB12, PB13, PB14},
        gpioc::PC13,
        Input, Output, PullDown, PushPull,
    },
    prelude::*,
    serial::{Rx, Serial, Tx},
    timer::{Event, Timer},
};

use glow::knob::{Direction, Knob};

const PERIOD: u32 = 8_000_000;

#[app(device = stm32f1::stm32f103)]
const APP: () = {
    static mut itm: cortex_m::peripheral::ITM = ();
    static mut toggle: bool = false;
    static mut led: PC13<Output<PushPull>> = ();
    static mut button: PB12<Input<PullDown>> = ();
    static mut knob: Knob<PB13<Input<PullDown>>, PB14<Input<PullDown>>> = ();

    //#[init(schedule = [foo])]
    #[init]
    fn init() -> init::LateResources {
        let rcc = device.RCC;
        let afio = device.AFIO;
        let exti = device.EXTI;

        rcc.apb2enr.modify(|_r, w| w.afioen().set_bit());
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
        let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
        let mut gpioc = device.GPIOC.split(&mut rcc.apb2);

        let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        let button = gpiob.pb12.into_pull_down_input(&mut gpiob.crh);
        let ka = gpiob.pb13.into_pull_down_input(&mut gpiob.crh);
        let kb = gpiob.pb14.into_pull_down_input(&mut gpiob.crh);
        let knob = Knob::new(ka, kb);

        //schedule.foo(Instant::now() + PERIOD.cycles()).unwrap();
        let itm = core.ITM;
        init::LateResources {
            itm,
            led,
            button,
            knob,
        }
    }

    #[interrupt(resources = [itm, toggle, led, button, knob])]
    fn EXTI15_10() {
        /*
        if resources.button.is_high() {
            *resources.toggle = true;
            resources.led.set_high();
        } else {
            *resources.toggle = false;
            resources.led.set_low();
        }
        */
        use Direction::*;
        match resources.knob.poll() {
            Some(CW) => resources.led.set_high(),
            Some(CCW) => resources.led.set_low(),
            None => {}
        }
    }

    extern "C" {
        fn EXTI0();
    }
};
