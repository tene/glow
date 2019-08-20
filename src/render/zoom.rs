use core::fmt::Write;

use heapless::{consts, String, Vec};

use crate::hsv::{HSV, HUE_MAX};
use crate::knob::Direction;
use crate::m6::{Node, Region, Render};

pub struct Zoom {
    hue: i16,
    speed: i16,
    step: i16,
}

impl Zoom {
    pub const fn new() -> Self {
        let hue = 0;
        let speed = 20;
        let step = -64;
        Self { hue, speed, step }
    }
}

impl Render for Zoom {
    fn render(&self, n: &Node) -> (HSV, HSV) {
        use Region::*;
        let (sa, sb): (i16, i16) = match n.region {
            Center => (0, 0),
            Inner => (1, 3),
            Ray => (2, 5),
            Outer => (4, 6),
        };

        let a = HSV::new(self.hue + (sa * self.step), 0x60, 0x80);
        let b = HSV::new(self.hue + (sb * self.step), 0x60, 0x80);
        (a, b)
    }
    fn tick(&mut self) {
        let h = self.hue + self.speed;
        let h = ((h % HUE_MAX) + HUE_MAX) % HUE_MAX;
        self.hue = h;
    }
    fn debug(&self) -> Vec<String<consts::U16>, consts::U8> {
        let mut rv = Vec::new();
        let mut speed_s = String::new();
        let mut hue_s = String::new();
        let mut step_s = String::new();
        let _ = write!(speed_s, "speed: {}", self.speed);
        let _ = write!(hue_s, "hue: {}", self.hue);
        let _ = write!(step_s, "step: {}", self.step);
        let _ = rv.push(speed_s);
        let _ = rv.push(hue_s);
        let _ = rv.push(step_s);
        rv
    }

    fn knob1(&mut self, dir: Direction) {
        use Direction::*;
        match dir {
            CW => {
                self.speed += 1;
            }
            CCW => {
                self.speed -= 1;
            }
        }
    }
    fn knob2(&mut self, dir: Direction) {
        use Direction::*;
        match dir {
            CW => {
                self.step += 1;
            }
            CCW => {
                self.step -= 1;
            }
        }
    }
}
