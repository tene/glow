use core::mem::swap;
use smart_leds::RGB8;

pub const HUE_MAX: i16 = (256 * 6) - 1;

#[derive(Clone, Copy, Debug)]
pub struct HSV {
    pub h: u16,
    pub s: u8,
    pub v: u8,
}

impl HSV {
    pub const fn new(h: u16, s: u8, v: u8) -> Self {
        Self { h, s, v }
    }
    // From http://www.vagrearg.org/content/hsvrgb
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        let &Self { h, s, v } = self;
        if s == 0 {
            return (v, v, v);
        }
        let mut r: u8 = 0;
        let mut g: u8 = 0;
        let mut b: u8 = 0;
        let mut pr = &mut r;
        let mut pg = &mut g;
        let mut pb = &mut b;
        let sextant: u8 = (h >> 8) as u8;
        if sextant & 2 != 0 {
            swap(&mut pr, &mut pb);
        }
        if sextant & 4 != 0 {
            swap(&mut pg, &mut pb);
        }
        if sextant & 6 == 0 {
            if sextant & 1 == 0 {
                swap(&mut pr, &mut pg);
            }
        } else {
            if sextant & 1 != 0 {
                swap(&mut pr, &mut pg);
            }
        }
        *pg = v;
        let mut ww: u16 = v as u16 * (255 - s as u16);
        ww += 1;
        ww += ww >> 8;
        *pb = (ww >> 8) as u8;
        let h_frac: u32 = (h as u32) & 0xff;
        let mut d: u32 = match sextant & 1 {
            // slope down
            0 => v as u32 * ((255 << 8) - (s as u32 * h_frac)),
            1 => v as u32 * ((255 << 8) - (s as u32 * (256 - h_frac))),
            _ => unreachable!(),
        };
        d += d >> 8;
        d += v as u32;
        *pr = (d >> 16) as u8;
        (r, g, b)
    }
    pub fn shift_hue(&mut self, d: i16) {
        let mut hue = self.h as i16 + d;
        if d.is_negative() {
            while hue < 0 {
                hue += HUE_MAX;
            }
        } else {
            hue %= HUE_MAX;
        }
        self.h = hue as u16;
    }
    pub fn shifted_hue(&self, d: i16) -> Self {
        let mut next = self.clone();
        next.shift_hue(d);
        next
    }
}

impl Into<RGB8> for HSV {
    fn into(self) -> RGB8 {
        self.to_rgb().into()
    }
}