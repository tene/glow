use core::{fmt::Write, ops::Add, ops::Mul};

use heapless::{consts, String};

use crate::hsv::{HSV, HUE_MAX};
use crate::knob::Direction;
use crate::m6::{Node, Region, Render};

pub struct Rainbow {
    offset: i16,
    speed: i16,
}

impl Rainbow {
    pub const fn new() -> Self {
        let offset = 0;
        let speed = 10;
        Self { offset, speed }
    }

    pub fn debug(&self) -> String<consts::U16> {
        let mut rv = String::new();
        let _ = write!(rv, "{} {}", self.speed, self.offset);
        rv
    }

    pub fn knob1(&mut self, dir: Direction) {
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
    pub fn knob2(&mut self, dir: Direction) {
        use Direction::*;
        match dir {
            CW => {}
            CCW => {}
        }
    }
}

impl Render for Rainbow {
    fn render(&self, n: &Node) -> (HSV, HSV) {
        use num_rational::Ratio;
        use Region::*;
        let ao: Ratio<i16> = match n.region {
            Center => Ratio::new(0, 12),
            Inner => Ratio::new(0, 12),
            Ray => Ratio::new(0, 12),
            Outer => Ratio::new(0, 12),
        };
        let hue = n.angle.add(ao).mul(HUE_MAX).to_integer() as i16;

        let a = HSV::new(self.offset + hue, 0xff, 0x80);
        (a, a)
    }
    fn tick(&mut self) {
        self.offset += self.speed;
    }
}
