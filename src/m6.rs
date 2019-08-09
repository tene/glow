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
    pub angle: Ratio<i16>,
}

fn build_nodes() -> Vec<Node, consts::U19> {
    use Region::*;
    let center = once(Node {
        region: Center,
        angle: Ratio::new(0, 12),
    });
    let petals = (0..6).map(|n| Node {
        region: Inner,
        angle: Ratio::new(n * 2, 12),
    });
    let asdf: [i16; 6] = [5, 0, 1, 2, 3, 4];
    let rays = asdf.iter().map(|n| Node {
        region: Ray,
        angle: Ratio::new((n * 2) + 1, 12),
    });
    let outer = asdf.iter().map(|n| Node {
        region: Outer,
        angle: Ratio::new(n * 2, 12),
    });
    center.chain(petals).chain(rays).chain(outer).collect()
}

lazy_static! {
    static ref NODES: Vec<Node, consts::U19> = build_nodes();
}

// TODO -> AsRef<RGB>
pub trait Render {
    fn render(&self, n: &Node) -> (HSV, HSV);
}

pub struct Generator<'a, T: Render> {
    idx: usize,
    carry: Option<HSV>,
    r: &'a T,
}

pub fn generate<'a, T: Render>(r: &'a T) -> Generator<'a, T> {
    let idx = 0;
    let carry = None;
    Generator { idx, carry, r }
}

impl<'a, T: Render> Iterator for Generator<'a, T> {
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
