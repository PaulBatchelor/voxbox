// TODO Make this a generic
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
