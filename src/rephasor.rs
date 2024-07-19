pub struct RePhasor {
    // main phasor state
    pr: f32,

    // last two 2 samples from correction phasor
    pc: [f32; 2],

    // last 2 samples from external phasor
    pe: [f32; 2],

    /// c: "course correction"
    c: f32,

    /// s: scaling amount for rephasor
    s: f32,

    /// si: inverse scale, or 1/s.
    si: f32,

    /// ir: increment amount of rephasor
    ir: f32,

    /// ic: increment amount for correction
    /// rephasor
    ic: f32,
}

impl Default for RePhasor {
    fn default() -> Self {
        RePhasor::new()
    }
}

impl RePhasor {
    pub fn new() -> Self {
        RePhasor {
            pr: 0.0,
            pc: [0.0; 2],
            pe: [0.0; 2],
            c: 1.0,
            s: 1.0,
            si: 1.0,
            ir: 0.0,
            ic: 0.0,
        }
    }

    pub fn set_scale(&mut self, scale: f32) {
        if scale != self.s {
            self.s = scale;
            self.si = 1.0 / scale;
        }
    }

    pub fn tick(&mut self, ext: f32) -> f32 {
        // compute increment of rephasor if it
        // is not the start of a new period
        if ext > self.pe[0] {
            self.ir = ext - self.pe[0];
        }

        // compute main rephasor theta_r
        let pr = phasor(self.pr, self.s * self.ir * self.c);

        // Create a "correction" rephasor. Feed
        // the output of the main into another
        // rephasor and undo the scaling.
        if pr > self.pr {
            self.ic = pr - self.pr;
        }

        // compute correction phasor
        let pc = phasor(self.pc[0], self.si * self.ic);

        // compute correction coefficient
        // Measure differences between the the
        // external phasor and the correction
        // phasor.
        if self.pc[1] != 0.0 {
            self.c = self.pe[1] / self.pc[1];
        }

        if self.c > 2.0 || self.c < 0.5 {
            self.c = 1.0
        }

        let out = pr;

        // update state
        self.pr = pr;
        self.pc[1] = self.pc[0];
        self.pc[0] = pc;

        self.pe[1] = self.pe[0];
        self.pe[0] = ext;

        out
    }

    pub fn reset(&mut self) {
        //self.process_ext = false;
        for i in 1..2 {
            self.pc[i] = 0.;
            self.pe[i] = 0.;
        }
        self.c = 1.0;
        self.s = 1.0;
        self.si = 1.0;
        self.pr = 0.0;
        self.ir = 0.0;
        self.ic = 0.0;
    }
}

// truncated phasor

fn phasor(phs: f32, inc: f32) -> f32 {
    let phs = phs + inc;

    if phs > 1.0 {
        return 0.0;
    }

    phs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_reset() {
        let sr = 44100;
        let mut rephasor = RePhasor::new();

        let mut lpsig = -1.0;
        let mut count = 0;
        let mut phs = 0.;
        let phs_inc = 440. / sr as f32;

        for _ in 1..sr * 5 {
            let mut psig = phasor(phs, phs_inc);

            if lpsig >= 0. && lpsig > psig {
                count += 1;
            }

            // reset halfway through period 2
            if count == 2 && phs > 0.5 {
                psig = 0.;
                rephasor.reset();
                count += 1;
            }

            rephasor.tick(psig);

            // Compare delta times of phasor/rephasor
            // They should approximtely match
            if lpsig >= 0. {
                let rp_delta = rephasor.s * rephasor.ir * rephasor.c;
                let diff = (rp_delta - phs_inc).abs();
                assert!(diff < 0.001, "RePhasor did not handle reset properly");
            }

            // compare delta increment values

            phs = psig;
            lpsig = psig;
        }
    }
}
