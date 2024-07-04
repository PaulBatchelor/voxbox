pub struct TimingParam {
    cur: f32,
    prev: f32,
    cached: f32,
}

enum EnvelopeState {
    Attack,
    Release
}

pub struct Envelope {
    y: f32,
    sr: usize,
    atk: TimingParam,
    rel: TimingParam,
    state: EnvelopeState,
    pgate: f32,
}

impl TimingParam {
    pub fn new() -> Self {
        TimingParam{
            cur: 0.1,
            prev: -100., cached: 0.0
        }
    }

    pub fn set_time(&mut self, time: f32) {
        self.cur = time;
    }

    pub fn update(&mut self, sr: usize) {
        if self.cur != self.prev {
            self.prev = self.cur;
            let tau = (self.cur / (1000.0_f32).ln()) * sr as f32;

            if tau > 0. {
                self.cached = (-1.0/tau).exp();
            }
        }
    }
}

impl Envelope {
    pub fn new(sr: usize) -> Self {
        Envelope {
            y: 0.0,
            sr: sr,
            atk: TimingParam::new(),
            rel: TimingParam::new(),
            state: EnvelopeState::Attack,
            pgate: -1.,
        }
    }

    pub fn set_attack(&mut self, time: f32) {
        self.atk.set_time(time);
    }

    pub fn set_release(&mut self, time: f32) {
        self.rel.set_time(time);
    }

    pub fn tick(&mut self, gate: f32) -> f32 {
        if gate > 0.5 && self.pgate <= 0.5 {
            self.state = EnvelopeState::Attack;
        } else if gate < 0.5 && self.pgate >= 0.5 {
            self.state = EnvelopeState::Release;
        }

        let p = match self.state {
            EnvelopeState::Attack => &mut self.atk,
            EnvelopeState::Release => &mut self.rel,
        };

        p.update(self.sr);

        let a1 = p.cached;
        let b0 = 1.0 - a1;
        let y = self.y;

        let out = b0 * gate + a1*y;

        self.y = out;
        self.pgate = gate;
        out
    }
}
