use std::f32::consts::PI;
use std::f32::consts::SQRT_2;

#[derive(Clone, Copy)]
pub struct ButterworthFilter {
    pidsr: f32,
    a: [f32; 64],
}

// TODO use traits here instead of nested structs?
#[derive(Clone, Copy)]
pub struct ButterworthLowPass {
    freq: f32,
    pfreq: f32,
    butter: ButterworthFilter,
}

// TODO use traits here instead of nested structs?
#[derive(Clone, Copy)]
pub struct ButterworthHighPass {
    freq: f32,
    pfreq: f32,
    butter: ButterworthFilter,
}

impl ButterworthFilter {
    pub fn new(sr: usize) -> Self {
        ButterworthFilter {
            a: [0.0; 64],
            pidsr: PI / sr as f32,
        }
    }

    fn filter(&mut self, insig: f32) -> f32 {
        let a = &mut self.a;
        let t = insig - a[3] * a[5] - a[4] * a[6];
        let y = t * a[0] + a[1] * a[5] + a[2] * a[6];
        a[6] = a[5];
        a[5] = t;
        y
    }
}

// TODO: use traits instead?
impl ButterworthLowPass {
    pub fn new(sr: usize) -> Self {
        ButterworthLowPass {
            freq: 1000.0,
            pfreq: -1.0,
            butter: ButterworthFilter::new(sr),
        }
    }

    pub fn set_freq(&mut self, f: f32) {
        self.freq = f;
    }

    pub fn tick(&mut self, insig: f32) -> f32 {
        if self.freq != self.pfreq {
            self.pfreq = self.freq;
            let a = &mut self.butter.a;

            // derive c constant for BLT
            let c = 1.0 / (self.freq * self.butter.pidsr).tan();

            a[0] = 1.0 / (1.0 + c * SQRT_2 + c * c);
            a[1] = 2.0 * a[0];
            a[2] = a[0];
            a[3] = 2.0 * (1.0 - c * c) * a[0];
            a[4] = (1.0 - c * SQRT_2 + c * c) * a[0];
        }

        self.butter.filter(insig)
    }
}

// TODO: use traits instead?
impl ButterworthHighPass {
    pub fn new(sr: usize) -> Self {
        ButterworthHighPass {
            freq: 1000.0,
            pfreq: -1.0,
            butter: ButterworthFilter::new(sr),
        }
    }

    pub fn set_freq(&mut self, f: f32) {
        self.freq = f;
    }

    pub fn tick(&mut self, insig: f32) -> f32 {
        if self.freq != self.pfreq {
            self.pfreq = self.freq;
            let a = &mut self.butter.a;

            // derive c constant for BLT
            let c = (self.freq * self.butter.pidsr).tan();

            a[0] = 1.0 / (1.0 + c*SQRT_2 + c*c);
            a[1] = -2.0*a[0];
            a[2] = a[0];
            a[3] = 2.0 * (c*c - 1.0) * a[0];
            a[4] = (1.0 - c*SQRT_2 + c*c) * a[0];
        }

        self.butter.filter(insig)
    }
}
