use core::mem::swap;
use smart_leds::RGB8;

pub const HUE_MAX: i16 = (256 * 6) - 1;

pub const GAMMA: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5,
    5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 13, 13, 13, 14,
    14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 24, 24, 25, 25, 26, 27,
    27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35, 36, 37, 38, 39, 39, 40, 41, 42, 43, 44, 45, 46,
    47, 48, 49, 50, 50, 51, 52, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 66, 67, 68, 69, 70, 72,
    73, 74, 75, 77, 78, 79, 81, 82, 83, 85, 86, 87, 89, 90, 92, 93, 95, 96, 98, 99, 101, 102, 104,
    105, 107, 109, 110, 112, 114, 115, 117, 119, 120, 122, 124, 126, 127, 129, 131, 133, 135, 137,
    138, 140, 142, 144, 146, 148, 150, 152, 154, 156, 158, 160, 162, 164, 167, 169, 171, 173, 175,
    177, 180, 182, 184, 186, 189, 191, 193, 196, 198, 200, 203, 205, 208, 210, 213, 215, 218, 220,
    223, 225, 228, 231, 233, 236, 239, 241, 244, 247, 249, 252, 255,
];

#[derive(Clone, Copy, Debug)]
pub struct HSV {
    pub h: u16,
    pub s: u8,
    pub v: u8,
}

impl HSV {
    pub const fn new(h: i16, s: u8, v: u8) -> Self {
        let h: u16 = (((h % HUE_MAX) + HUE_MAX) % HUE_MAX) as u16;
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
        //(r, g, b)
        (GAMMA[r as usize], GAMMA[g as usize], GAMMA[b as usize])
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

impl Into<RGB8> for &HSV {
    fn into(self) -> RGB8 {
        self.to_rgb().into()
    }
}
