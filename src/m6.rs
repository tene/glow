use heapless::{consts, Vec};
use lazy_static::lazy_static;
use num_rational::Ratio;

use crate::hsv::HSV;

use core::iter::once;

#[derive(Clone, Copy, Debug)]
pub enum Region {
    Center,
    Inner,
    Ray,
    Outer,
}

#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub region: Region,
    pub angle: Ratio<u16>,
}

fn build_nodes() -> Vec<Node, consts::U19> {
    use Region::*;
    let center = Node {
        region: Center,
        angle: Ratio::new(0, 12),
    };
    once(center).cycle().take(19).collect()
}

lazy_static! {
    static ref NODES: Vec<Node, consts::U19> = build_nodes();
}

// TODO -> AsRef<RGB>
pub type ColorFn<'a> = &'a mut dyn FnMut(&Node) -> (HSV, HSV);

pub struct Generator<'a> {
    idx: usize,
    carry: Option<HSV>,
    f: ColorFn<'a>,
}

pub fn generate(f: ColorFn) -> Generator {
    let idx = 0;
    let carry = None;
    Generator { idx, carry, f }
}

impl<'a> Iterator for Generator<'a> {
    type Item = HSV;
    fn next(&mut self) -> Option<HSV> {
        let carry = self.carry.take();
        if carry.is_some() {
            return carry;
        }
        if self.idx >= NODES.len() {
            return None;
        }
        let (rv, extra) = (self.f)(&NODES[self.idx]);
        self.idx += 1;
        self.carry = Some(extra);
        Some(rv)
    }
}
