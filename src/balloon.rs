pub struct Balloon {
    pub pressure: f32,
    pub deflation: f32,
    pub inflation: f32,
    p_deflation: f32,
    p_inflation: f32,
    volume: f32,
    sr: usize,
    inflate: f32,
    deflate: f32,
    pub infinity_mode: bool,
}

fn tau2pole(tau: f32, sr: usize) -> f32 {
    let tau = tau / (1000_f32).ln();
    let tau = tau * sr as f32;

    if tau <= 0. {
        return 0.;
    }

    (-1.0 / tau).exp()
}

impl Balloon {
    pub fn new(sr: usize) -> Self {
        Balloon {
            volume: 0.,
            sr,
            pressure: 0.,
            inflation: 0.5,
            deflation: 1.0,

            // assume only positive values
            p_inflation: -1.0,
            p_deflation: -1.0,
            inflate: 0.,
            deflate: 1.0,
            infinity_mode: false,
        }
    }

    pub fn tick(&mut self) -> f32 {
        if self.inflation != self.p_inflation {
            self.p_inflation = self.inflation;

            //let tau = self.inflation / (1000_f32).ln();
            //let tau = tau * self.sr as f32;

            //if tau > 0. {
            //    self.inflate = (-1.0 / tau).exp();
            //}
            self.inflate = tau2pole(self.inflation, self.sr);
        }

        if self.deflation != self.p_deflation {
            self.p_deflation = self.deflation;

            if self.infinity_mode {
                self.deflate = 1.0;
            } else {
                self.deflate = tau2pole(self.deflation, self.sr);
            }
        }

        // pressure is a scalar that determines the overall rate

        let out = self.volume;

        let mut inflate = 1.0 - self.inflate;
        inflate *= self.pressure;

        if inflate < 0. {
            inflate = 0.;
        }

        let mut vol = self.volume;
        vol = inflate + (1.0 - inflate) * vol;

        // truncate, if needed (can happen if pressure > 1.0

        if vol > 1.0 {
            vol = 1.0;
        }

        vol *= self.deflate;

        self.volume = vol;
        out
    }
}
