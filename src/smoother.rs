#[derive(Clone, Copy)]
pub struct Smoother {
    smooth: f32,
    psmooth: f32,
    a1: f32,
    b0: f32,
    y0: f32,
    onedsr: f32,
}

impl Smoother {
    pub fn new(sr: usize) -> Self {
        Smoother {
            smooth: 0.01,
            psmooth: -1.0,
            a1: 0.0,
            b0: 0.0,
            y0: 0.0,
            onedsr: 1.0 / sr as f32,
        }
    }

    pub fn set_smooth(&mut self, smooth: f32) {
        self.smooth = smooth;
    }

    pub fn snap_to_value(&mut self, value: f32) {
        self.y0 = value;
    }

    pub fn tick(&mut self, sig: f32) -> f32 {

        if self.psmooth != self.smooth {
            self.a1 = (0.5_f32).powf(self.onedsr / self.smooth);
            self.b0 = 1.0 - self.a1;
            self.psmooth = self.smooth;
        }
        self.y0 = self.b0*sig + self.a1*self.y0;
        self.y0
    }

}
