use voxbox::*;

fn gliss_it(phs: f32, glisspos: f32) -> f32 {
    let mut a;
    if phs < glisspos {
        a = 0.0;
    } else {
        a = phs - glisspos;
        if a < 0.0 {
            a = 0.0;
        }
        a /= 1.0 - glisspos;
        a = a * a * a;
    }
    a
}

struct ChatterBox {
    voice: Voice,
    shape_morpher: RandomPhasor,
    chooser: LinearCongruentialGenerator,
    drm: [f32; 8],
    jit_freq: Jitter,
    tgate: TriggerGate,
    env: Envelope,
    shapes: Vec<[f32; 8]>,
    cur: usize,
    nxt: usize,
    pphs: f32,
    gate: u8,
    pgate: u8,
    pressure: Envelope,
    pressure_gate: TriggerGate,
    balloon: Balloon,
}

impl ChatterBox {
    pub fn new(sr: usize) -> Self {
        let oversample = 2;
        let tract_len = 8.7;
        let mut cb = ChatterBox {
            voice: Voice::new(sr, tract_len, oversample),
            shape_morpher: RandomPhasor::new(sr, 0.),
            chooser: LinearCongruentialGenerator::new(),
            drm: [0.5; 8],
            jit_freq: Jitter::new(sr),
            tgate: TriggerGate::new(sr),
            env: Envelope::new(sr),
            shapes: generate_shape_table(),
            pphs: -1.,
            cur: 0,
            nxt: 1,
            gate: 0,
            pgate: 0,
            pressure: Envelope::new(sr),
            balloon: Balloon::new(sr),
            pressure_gate: TriggerGate::new(sr),
        };

        cb.shape_morpher.min_freq = 3.0;
        cb.shape_morpher.max_freq = 10.0;
        cb.chooser.seed(4444);
        cb.jit_freq.seed(43438, 5555);
        cb.jit_freq.range_amplitude(-2., 2.);
        cb.jit_freq.range_rate(3., 10.);
        cb.tgate.duration = 0.4;
        cb.env.set_attack(0.01);
        cb.env.set_release(0.7);
        cb.voice.pitch = 63.;
        cb.pressure.set_attack(0.01);
        cb.pressure.set_release(0.05);
        cb.balloon.inflation = 3.0;
        cb.balloon.deflation = 0.5;
        cb.pressure_gate.duration = 0.1;
        cb
    }

    pub fn poke(&mut self) {
        self.gate = !self.gate;
    }

    pub fn tick(&mut self) -> f32 {
        let voice = &mut self.voice;
        let shape_morpher = &mut self.shape_morpher;

        let chooser = &mut self.chooser;
        let drm = &mut self.drm;
        let jit_freq = &mut self.jit_freq;
        let tgate = &mut self.tgate;
        let env = &mut self.env;

        let shapes = &mut self.shapes;
        let cur = &mut self.cur;
        let nxt = &mut self.nxt;
        let pphs = &mut self.pphs;
        let gate = &self.gate;
        let pgate = &mut self.pgate;

        let phs = shape_morpher.tick();
        if phs < *pphs {
            *cur = *nxt;
            *nxt = (chooser.randf() * shapes.len() as f32) as usize;
            *cur %= shapes.len();
            *nxt %= shapes.len();
        }
        let shp_a = shapes[*cur];
        let shp_b = shapes[*nxt];
        let t = if *pgate != *gate { 1.0 } else { 0.0 };
        let gt = tgate.tick(t);
        let pg = self.pressure_gate.tick(t);
        let pr = self.pressure.tick(pg);
        self.balloon.pressure = pr * 1.1;
        let bal = self.balloon.tick();

        let jf = jit_freq.tick();
        voice.pitch = 63.0 + jf * bal + 12. * bal;
        //voice.pitch = 63.0 + 7. * bal;

        let alpha = gliss_it(phs, 0.8);
        for i in 0..8 {
            drm[i] = (1. - alpha) * shp_a[i] + alpha * shp_b[i];
        }

        voice.tract.drm(drm);

        let ev = env.tick(gt);

        //let out = voice.tick() * 0.5 * ev;
        let out = voice.tick() * 0.5;
        *pphs = phs;
        *pgate = *gate;
        out
    }
}

fn generate_shape_table() -> Vec<[f32; 8]> {
    let tiny_ah = [0.77, 0.855, 1.435, 0.728, 1.067, 3.217, 0.671, 2.892];
    let tiny_ieh = [0.6, 1.081, 4., 3.741, 0.954, 0.572, 0.487, 1.704];
    let tiny_r4mod1 = [0.53, 1.435, 0.303, 3.798, 2.383, 0.374, 2.807, 0.685];
    let tiny_r4mod2 = [0.53, 1.435, 0.303, 0.1, 2.383, 0.374, 2.807, 0.685];
    let shapes = vec![tiny_ah, tiny_ieh, tiny_r4mod2, tiny_r4mod1];

    shapes
}

fn main() {
    let sr = 44100;
    let mut cb = ChatterBox::new(sr);
    let mut wav = MonoWav::new("chatter.wav");
    let mut metro = Metro::new(sr);

    let mut jit_rate = Jitter::new(sr);

    jit_rate.range_rate(1.0, 4.0);
    jit_rate.range_amplitude(0.8, 2.0);

    for _ in 0..(sr as f32 * 10.0) as usize {
        metro.set_rate(jit_rate.tick());
        let trig = metro.tick();

        if trig > 0. {
            cb.poke();
        }
        wav.tick(cb.tick());
    }
}
