// TODO Make this a generic?
pub struct LinearCongruentialGenerator {
    rng: u32,
}

impl LinearCongruentialGenerator {
    pub fn new() -> Self {
        LinearCongruentialGenerator {
            rng: 0,
        }
    }

    pub fn rand(&mut self) -> u32 {
        self.rng = self.rng.wrapping_mul(1103515245);
        self.rng = self.rng.wrapping_add(12345) % u32::MAX;
        return self.rng;
    }

    pub fn randf(&mut self) -> f32 {
        return self.rand() as f32 / u32::MAX as f32;
    }

    pub fn seed(&mut self, val: u32) {
        self.rng = val;
    }
}

pub struct RandomLine {
    rng: LinearCongruentialGenerator,
    pub min: f32,
    pub max: f32,
    lphs: f32,
    val_a: f32,
    val_b: f32,
}

impl RandomLine {
    pub fn new() -> Self {
        RandomLine {
            lphs: -1.0,
            val_a: 0.0,
            val_b: 0.0,
            min: 0.0,
            max: 1.0,
            rng: LinearCongruentialGenerator::new(),
        }
    }

    pub fn seed(&mut self, val: u32) {
        self.rng.seed(val);
    }

    pub fn tick(&mut self, phs: f32) -> f32 {
        let lphs = self.lphs;

        if lphs < 0.  {
            // (re-)initialize generator, generate
            // two new random values

            self.val_a = self.rng.randf();
            self.val_b = self.rng.randf();
        } else if phs < lphs {
            // New Period. Update points.
            
            self.val_a = self.val_b;
            self.val_b = self.rng.randf();
        }


        // scale and interpolate
        let diff = self.max - self.min;
        let min = self.min;
        let a = self.val_a*diff + min;
        let b = self.val_b*diff + min;

        let out = (1.0 - phs)*a + phs*b;

        self.lphs = phs;

        out
    }
}


// TODO copy-pasted from Phasor. De-duplication?
pub struct RandomPhasor {
    phs: f32,
    onedsr: f32,
    rng: LinearCongruentialGenerator,
    pub min_freq: f32,
    pub max_freq: f32,
    pub rval: f32,
}

impl RandomPhasor {
    pub fn new(sr: usize, iphs: f32) -> Self {
        RandomPhasor {
            phs: iphs,
            onedsr: 1.0 / sr as f32,
            rng: LinearCongruentialGenerator::new(),
            min_freq: 1.,
            max_freq: 1.,
            rval: -1.,
        }
    }

    pub fn seed(&mut self, val: u32) {
        self.rng.seed(val);
    }

    pub fn tick(&mut self) -> f32 {

        if self.rval < 0. {
            self.rval = self.rng.randf();
        }

        let freq =
            (self.max_freq - self.min_freq)*
            self.rval +
            self.min_freq;
        let incr = freq * self.onedsr;
        let mut phs = self.phs;

        let out = phs;

        phs += incr;

        if phs >= 1.0 {
            phs -= 1.0;
            self.rval = -1.;
        } else if phs < 0.0 {
            phs += 1.0;
            self.rval = -1.;
        }

        self.phs = phs;

        out
    }

}

pub struct Jitter {
    phasor: RandomPhasor,
    linseg: RandomLine,
}

impl Jitter {
    pub fn new(sr: usize) -> Self {
        Jitter {
            phasor: RandomPhasor::new(sr, 0.),
            linseg: RandomLine::new(),
        }
    }

    pub fn range_rate(&mut self, min: f32, max: f32) {
        self.phasor.min_freq = min;
        self.phasor.max_freq = max;
    }

    pub fn range_amplitude(&mut self, min: f32, max: f32) {
        self.linseg.min = min;
        self.linseg.max = max;
    }

    pub fn tick(&mut self) -> f32 {
        let phs = self.phasor.tick();
        let out = self.linseg.tick(phs);
        out
    }

    pub fn seed(&mut self, lseed: u32, pseed: u32) {
        self.linseg.seed(lseed);
        self.phasor.seed(pseed);
    }

}
