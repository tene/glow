use heapless::{consts, String, Vec};
use lazy_static::lazy_static;
use num_rational::Ratio;

use crate::hsv::HSV;
use crate::knob::Direction;

use core::iter::once;

#[derive(Clone, Copy, Debug)]
pub enum Region {
    Center,
    Inner,
    Ray,
    Outer,
}

impl Region {
    pub fn r(&self) -> usize {
        use Region::*;
        match self {
            Center => 0,
            Inner => 1,
            Ray => 2,
            Outer => 3,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub region: Region,
    pub angle: Ratio<i16>,
    pub idx: i16,
}

fn build_nodes() -> Vec<Node, consts::U19> {
    use Region::*;
    let center = once(Node {
        region: Center,
        angle: Ratio::new(0, 12),
        idx: 0,
    });
    let petals = (0..6).map(|n| Node {
        region: Inner,
        angle: Ratio::new(n * 2, 12),
        idx: n,
    });
    let asdf: [i16; 6] = [5, 0, 1, 2, 3, 4];
    let rays = asdf.iter().map(|n| Node {
        region: Ray,
        angle: Ratio::new((n * 2) + 1, 12),
        idx: *n,
    });
    let outer = asdf.iter().map(|n| Node {
        region: Outer,
        angle: Ratio::new(n * 2, 12),
        idx: *n,
    });
    center.chain(petals).chain(rays).chain(outer).collect()
}

lazy_static! {
    static ref NODES: Vec<Node, consts::U19> = build_nodes();
}

// TODO -> AsRef<RGB>
pub trait Render {
    fn render(&self, n: &Node) -> (HSV, HSV);
    fn tick(&mut self) {}
    fn debug(&self) -> Vec<String<consts::U16>, consts::U8> {
        let rv = Vec::new();
        rv
    }

    fn knob1(&mut self, _dir: Direction) {}
    fn knob2(&mut self, _dir: Direction) {}
}

pub struct Generator<'a> {
    idx: usize,
    carry: Option<HSV>,
    r: &'a dyn Render,
}

impl<'a> Generator<'a> {
    pub fn new(r: &'a dyn Render) -> Self {
        let idx = 0;
        let carry = None;
        Self { idx, carry, r }
    }
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
        let (rv, extra) = self.r.render(&NODES[self.idx]);
        self.idx += 1;
        self.carry = Some(extra);
        Some(rv)
    }
}
