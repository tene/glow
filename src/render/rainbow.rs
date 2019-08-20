use core::{fmt::Write, ops::Add, ops::Mul};

use heapless::{consts, String, Vec};

use crate::hsv::{HSV, HUE_MAX};
use crate::knob::Direction;
use crate::m6::{Node, Region, Render};

pub struct Rainbow {
    offset: i16,
    speed: i16,
    saturation: u8,
}

impl Rainbow {
    pub const fn new() -> Self {
        let offset = 0;
        let speed = 10;
        let saturation = 0xff;
        Self {
            offset,
            speed,
            saturation,
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

        let a = HSV::new(self.offset + hue, self.saturation, 0x80);
        (a, a)
    }
    fn tick(&mut self) {
        self.offset += self.speed;
    }
    fn debug(&self) -> Vec<String<consts::U16>, consts::U8> {
        let mut rv = Vec::new();
        let mut speed_s = String::new();
        let mut offset_s = String::new();
        let mut sat_s = String::new();
        let _ = write!(speed_s, "speed: {}", self.speed);
        let _ = write!(offset_s, "offset: {}", self.offset);
        let _ = write!(sat_s, "saturation: {}", self.saturation);
        let _ = rv.push(speed_s);
        let _ = rv.push(offset_s);
        let _ = rv.push(sat_s);
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
                self.saturation += 1;
            }
            CCW => {
                self.saturation -= 1;
            }
        }
    }
}
