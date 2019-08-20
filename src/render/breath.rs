use core::fmt::Write;

use heapless::{consts, String, Vec};
use libm::F32Ext;

use crate::hsv::HSV;
use crate::knob::Direction;
use crate::m6::{Node, Region, Render};

pub struct Breath {
    hue: i16,
    phase: f32,
    speed: f32,
    scale: f32,
}

impl Breath {
    pub const fn new() -> Self {
        let hue = 0;
        let phase = 0.0;
        let speed = 0.1;
        let scale = 128.0;
        Self {
            hue,
            phase,
            speed,
            scale,
        }
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
        let (vma, vmb): (f32, f32) = match n.region {
            Center => (1.0, 1.0),
            Inner => (0.8, 0.4),
            Ray => (0.6, 0.2),
            Outer => (0.0, -0.2),
        };

        let b = breathe(self.phase);
        let size = 64.0;

        let a = HSV::new(
            self.hue + (self.scale * (1.0 - vma) * b) as i16,
            0xa0,
            128 + (size * vma * b) as u8,
        );
        let b = HSV::new(
            self.hue + (self.scale * (1.0 - vmb) * b) as i16,
            0xa0,
            128 + (size * vmb * b) as u8,
        );
        (a, b)
    }
    fn tick(&mut self) {
        self.phase += self.speed;
        self.phase %= core::f32::consts::PI * 2.0;
    }
    fn debug(&self) -> Vec<String<consts::U16>, consts::U8> {
        let mut rv = Vec::new();
        let mut speed_s = String::new();
        let mut hue_s = String::new();
        let mut scale_s = String::new();
        let _ = write!(speed_s, "speed: {}", (self.speed * 100.0) as i16);
        let _ = write!(hue_s, "hue: {}", (self.hue) as i16);
        let _ = write!(scale_s, "scale: {}", (self.scale) as i16);
        let _ = rv.push(speed_s);
        let _ = rv.push(hue_s);
        let _ = rv.push(scale_s);
        rv
    }

    fn knob1(&mut self, dir: Direction) {
        use Direction::*;
        match dir {
            CW => {
                self.hue += 8;
            }
            CCW => {
                self.hue -= 8;
            }
        }
    }
    fn knob2(&mut self, dir: Direction) {
        use Direction::*;
        match dir {
            CW => {
                self.scale += 8.0;
            }
            CCW => {
                self.scale -= 8.0;
            }
        }
    }
}
