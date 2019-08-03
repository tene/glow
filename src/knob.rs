use embedded_hal::digital::v2::InputPin;

pub enum Direction {
    CW,
    CCW,
}

pub struct Knob<A: InputPin, B: InputPin> {
    a: A,
    b: B,
    last: (bool, bool),
}

impl<A: InputPin, B: InputPin> Knob<A, B> {
    pub fn new(a: A, b: B) -> Self {
        let last = (false, false);
        Self { a, b, last }
    }
    #[inline(never)]
    pub fn poll(&mut self) -> Option<Direction> {
        let next: (bool, bool) = (
            self.a.is_high().unwrap_or(false),
            self.b.is_high().unwrap_or(false),
        );
        let last = self.last;
        self.last = next;
        use Direction::*;
        match (last, next) {
            ((false, false), (false, true)) => Some(CW),
            ((false, true), (true, true)) => Some(CW),
            ((true, true), (true, false)) => Some(CW),
            ((true, false), (false, false)) => Some(CW),
            ((true, false), (true, true)) => Some(CCW),
            ((true, true), (false, true)) => Some(CCW),
            ((false, true), (false, false)) => Some(CCW),
            ((false, false), (true, false)) => Some(CCW),
            _ => None,
        }
    }
}
