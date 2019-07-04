//#![deny(unsafe_code)]
//#![deny(warnings)]
#![no_main]
#![no_std]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust53964
extern crate panic_semihosting; // panic handler

use rtfm::app;
use stm32f1xx_hal::{
    gpio::{
        gpioa::{PA5, PA6, PA7},
        gpiob::{PB12, PB13, PB14},
        gpioc::PC13,
        Alternate, Floating, Input, Output, PullDown, PushPull, GpioExt,
    },
    afio::AfioExt,
    rcc::RccExt,
    flash::FlashExt,
    time::U32Ext,
    spi::Spi,
    stm32::SPI1,
};
use embedded_hal::digital::v2::OutputPin;
/*
+    prelude::{
+        _embedded_hal_Pwm, _embedded_hal_PwmPin, _stm32_hal_afio_AfioExt,
+        _stm32_hal_flash_FlashExt, _stm32_hal_gpio_GpioExt, _stm32_hal_rcc_RccExt,
+        _stm32_hal_time_U32Ext,
+    }
*/

#[allow(unused)] // NOTE(allow) bug rust-lang/rust53964
use apa102_spi::Apa102;
use smart_leds::{SmartLedsWrite, RGB8};
use ws2812_spi::Ws2812;

use glow::knob::{Direction, Knob};

#[app(device = stm32f1::stm32f103)]
const APP: () = {
    static mut toggle: bool = false;
    static mut led: PC13<Output<PushPull>> = ();
    static mut button: PB12<Input<PullDown>> = ();
    static mut knob: Knob<PB13<Input<PullDown>>, PB14<Input<PullDown>>> = ();
    //static mut led_strip: Apa102<
    static mut led_strip: Ws2812<
        Spi<
            SPI1,
            (
                PA5<Alternate<PushPull>>,
                PA6<Input<Floating>>,
                PA7<Alternate<PushPull>>,
            ),
        >,
    > = ();

    //#[init(schedule = [foo])]
    #[init]
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
            .sysclk(16.mhz())
            .freeze(&mut flash.acr);
        let mut afio = afio.constrain(&mut rcc.apb2);
        let mut gpioa = device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
        let mut gpioc = device.GPIOC.split(&mut rcc.apb2);

        let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        let button = gpiob.pb12.into_pull_down_input(&mut gpiob.crh);
        let ka = gpiob.pb13.into_pull_down_input(&mut gpiob.crh);
        let kb = gpiob.pb14.into_pull_down_input(&mut gpiob.crh);
        let knob = Knob::new(ka, kb);

        let pa5 = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
        let pa6 = gpioa.pa6.into_floating_input(&mut gpioa.crl);
        let pa7 = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

        let spi_pins = (pa5, pa6, pa7);
        let spi = Spi::spi1(
            device.SPI1,
            spi_pins,
            &mut afio.mapr,
            //apa102_spi::MODE,
            ws2812_spi::MODE,
            3_000_000.hz(),
            clocks,
            &mut rcc.apb2,
        );
        //let led_strip = Apa102::new(spi);
        let led_strip = Ws2812::new(spi);

        //schedule.foo(Instant::now() + PERIOD.cycles()).unwrap();
        init::LateResources {
            led,
            button,
            knob,
            led_strip,
        }
    }

    #[interrupt(resources = [led, button, knob, led_strip])]
    fn EXTI15_10() {
        let all_red: [RGB8; 8] = [(255u8, 128u8, 128u8).into(); 8];
        let all_blue: [RGB8; 8] = [(128u8, 128u8, 255u8).into(); 8];
        use Direction::*;
        match resources.knob.poll() {
            Some(CW) => {
                let _ = resources.led_strip.write(all_red.iter().cloned());
                let _ = resources.led.set_high();
            }
            Some(CCW) => {
                let _ = resources.led_strip.write(all_blue.iter().cloned());
                let _ = resources.led.set_low();
            }
            None => {}
        }
    }

    extern "C" {
        fn EXTI0();
    }
};
