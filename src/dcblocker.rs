pub struct DCBlocker {
    x: f32,
    y: f32,
    r: f32,
}

impl DCBlocker {
    pub fn new(_sr: usize) -> Self {
        DCBlocker {
            x: 0.0,
            y: 0.0,
            r: 0.99,
        }
    }

    pub fn tick(&mut self, sig: f32) -> f32 {
        self.y = sig - self.x + self.r*self.y;
        self.x = sig;
        return self.y;
    }

}
