use core::{fmt::Write};

use heapless::{consts, String, Vec};
use libm::F32Ext;

use crate::hsv::{HSV, HUE_MAX};
use crate::knob::Direction;
use crate::m6::{Node, Region, Render};

pub struct Breath {
    hue: i16,
    phase: f32,
    speed: f32,
}

impl Breath {
    pub const fn new() -> Self {
        let hue = 0;
        let phase = 0.0;
        let speed = 0.1;
        Self { hue, phase, speed }
    }
}

fn breathe(x: f32) -> f32 {
    use core::f32::consts::E;
    let scale: f32 = 1.0 / (E - (1.0 / E));
    ((x.sin().exp() - (1.0 / E)) * scale)
}

impl Render for Breath {
    fn render(&self, n: &Node) -> (HSV, HSV) {
        use Region::*;
        let (vma,vmb): (f32,f32) = match n.region {
            Center => (1.0,1.0),
            Inner => (0.8,0.6),
            Ray => (0.7,0.5),
            Outer => (0.4,0.3),
        };

        let b = breathe(self.phase);
        let size = 128.0;

        let a = HSV::new(self.hue + (96.0 * (1.0 - vma) * b) as i16, 0xff, 72 + (size * vma * b) as u8);
        let b = HSV::new(self.hue + (96.0 * (1.0 - vmb) * b) as i16, 0xff, 72 + (size * vmb * b) as u8);
        (a, b)
    }
    fn tick(&mut self) {
        self.phase += self.speed;
        self.phase %= core::f32::consts::PI * 2.0;
    }
    fn debug(&self) -> Vec<String<consts::U16>, consts::U8> {
        let mut rv = Vec::new();
        let mut speed_s = String::new();
        let mut offset_s = String::new();
        let _ = write!(speed_s, "speed: {}", self.speed);
        let _ = write!(offset_s, "phase: {}", self.phase);
        let _ = rv.push(speed_s);
        let _ = rv.push(offset_s);
        rv
    }

    fn knob1(&mut self, dir: Direction) {
        use Direction::*;
        match dir {
            CW => {
                self.speed += 0.1;
            }
            CCW => {
                self.speed -= 0.1;
            }
        }
    }
    fn knob2(&mut self, dir: Direction) {
        use Direction::*;
        match dir {
            CW => {}
            CCW => {}
        }
    }
}
