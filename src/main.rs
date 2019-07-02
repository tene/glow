#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust53964
extern crate panic_itm; // panic handler

use cortex_m::iprintln;
use rtfm::{app, Instant};

const PERIOD: u32 = 8_000_000;

#[app(device = stm32f1::stm32f103)]
const APP: () = {
    static mut itm: cortex_m::peripheral::ITM = ();
    #[init(schedule = [foo])]
    fn init() -> init::LateResources {
        schedule.foo(Instant::now() + PERIOD.cycles()).unwrap();
        init::LateResources { itm: core.ITM }
    }

    #[task(resources = [itm], schedule = [foo])]
    fn foo() {
        let now = Instant::now();
        iprintln!(&mut resources.itm.stim[0], "Tick {:?} {:?}", scheduled, now);
        schedule.foo(scheduled + PERIOD.cycles()).unwrap();
    }

    extern "C" {
        fn EXTI0();
    }
};
