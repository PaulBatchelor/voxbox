pub struct Phasor {
    freq: f32,
    phs: f32,
    onedsr: f32,
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

    pub fn reset(&mut self) {
        self.phs = 0.;
    }
}

pub struct PhasorTrig {
    lphs: f32,
}

impl Default for PhasorTrig {
    fn default() -> Self {
        PhasorTrig::new()
    }
}

impl PhasorTrig {
    pub fn new() -> Self {
        PhasorTrig { lphs: -1.0 }
    }

    pub fn tick(&mut self, phs: f32) -> f32 {
        let lphs = self.lphs;
        let out = if lphs < 0.0 || lphs > phs { 1.0 } else { 0.0 };

        self.lphs = phs;

        out
    }
}

pub struct Metro {
    phs: Phasor,
    trig: PhasorTrig,
}

impl Metro {
    pub fn new(sr: usize) -> Self {
        Metro {
            phs: Phasor::new(sr, 0.),
            trig: PhasorTrig::new(),
        }
    }

    pub fn tick(&mut self) -> f32 {
        let phs = self.phs.tick();
        self.trig.tick(phs)
    }

    pub fn set_rate(&mut self, rate: f32) {
        self.phs.set_freq(rate)
    }
}
