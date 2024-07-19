pub struct RePhasor {
    // These were once math constants... I think.
    // I copied them over from the sndkit implementation
    pr: f32,
    pc: [f32; 2],
    pe: [f32; 2],
    c: f32,
    s: f32,
    si: f32,

    ir: f32,
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

    // truncated phasor

    pub fn tick(&mut self, ext: f32) -> f32 {
        // delta function of theta_e

        if ext > self.pe[0] {
            self.ir = ext - self.pe[0];
        }

        // compute main rephasor theta_r
        let pr = phasor(self.pr, self.s * self.ir * self.c);

        // delta function of theta_r

        if pr > self.pr {
            self.ic = pr - self.pr;
        }

        // compute rephasor theta_c

        let pc = phasor(self.pc[0], self.si * self.ic);

        // compute correction coefficient
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
}

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
                // TODO: reset period
                psig = 0.;
            }

            rephasor.tick(psig);

            // Compare delta times of phasor/rephasor
            // They should approximtely match
            if lpsig >= 0. {
                let rp_delta = rephasor.s * rephasor.ir * rephasor.c;
                assert!(
                    (rp_delta - phs_inc).abs() < 0.001,
                    "RePhasor did not handle reset properly"
                );
            }

            // compare delta increment values

            phs = psig;
            lpsig = psig;
        }
    }
}
