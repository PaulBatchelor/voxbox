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
    ic: f32
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

        let pc = phasor(self.pc[0], self.si*self.ic);

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

