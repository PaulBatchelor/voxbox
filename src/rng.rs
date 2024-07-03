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
    pub fn new(&mut self) -> Self {
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
