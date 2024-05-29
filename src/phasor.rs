pub struct Phasor {
    freq: f32,
    phs: f32,
    onedsr: f32 ,
}

impl Phasor {
    pub fn new(sr: usize, iphs: f32) -> Self {
        Phasor {
            freq: 1000.0,
            phs: iphs,
            onedsr: 1.0 / sr as f32,
        }
    }

    pub fn set_freq(&mut self, f: f32) {
        self.freq = f;
    }

    pub fn tick(&mut self) -> f32 {
        let incr = self.freq * self.onedsr;
        let mut phs = self.phs;

        let out = phs;

        phs += incr;

        if phs >= 1.0 {
            phs -= 1.0;
        } else if phs < 0.0 {
            phs += 1.0;
        }

        self.phs = phs;

        out
    }

}
