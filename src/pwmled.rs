use embedded_hal::PwmPin;
use stm32f1xx_hal::{
    pac::TIM4,
    pwm::{Pwm, C1, C2, C4},
};

pub struct PwmLed {
    max: u16,
    r: Pwm<TIM4, C1>,
    g: Pwm<TIM4, C2>,
    b: Pwm<TIM4, C4>,
}

impl PwmLed {
    pub fn new(mut r: Pwm<TIM4, C1>, mut g: Pwm<TIM4, C2>, mut b: Pwm<TIM4, C4>) -> Self {
        let max = r.get_max_duty();
        r.enable();
        g.enable();
        b.enable();
        Self { max, r, g, b }
    }
    pub fn rgb8(&mut self, r: u8, g: u8, b: u8) {
        let i = self.max / 256;
        self.r.set_duty(i * (r as u16));
        self.g.set_duty(i * (g as u16));
        self.b.set_duty(i * (b as u16));
    }
    pub fn rgb_f32(&mut self, r: f32, g: f32, b: f32) {
        let max = self.max as f32;
        self.r.set_duty((max * r) as u16);
        self.g.set_duty((max * g) as u16);
        self.b.set_duty((max * b) as u16);
    }
}
