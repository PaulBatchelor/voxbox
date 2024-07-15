use std::f32::consts::PI;
use voxbox::*;

struct Voice {
    pub glot: Glot,
    pub tract: Tract,
}

impl Voice {
    pub fn new(sr: usize, length: f32) -> Self {
        let mut v = Voice {
            glot: Glot::new(sr),
            tract: Tract::new(sr, length, 2),
        };
        v.glot.set_shape(0.4);
        v.glot.set_aspiration(0.3);
        v.glot.set_noise_floor(0.287);
        v
    }

    pub fn tick(&mut self) -> f32 {
        let s = self.glot.tick() * 0.7;
        let t = self.tract.tick(s);

        t
    }
}

// midi-to-frequency converter
pub fn mtof(nn: f32) -> f32 {
    let freq = (2.0_f32).powf((nn - 69.0) / 12.0) * 440.0;
    freq
}

// a simple sine wave generator
pub fn sin(frq: f32, n: usize, tpidsr: f32) -> f32 {
    let lfo = (frq * n as f32 * tpidsr).sin();
    let lfo = (lfo + 1.0) * 0.5;
    lfo
}

fn main() {
    let sr = 44100;

    // write to wav
    let mut wav = MonoWav::new("rephasoring.wav");

    let mut voice1 = Voice::new(sr, 15.0);
    let mut voice2 = Voice::new(sr, 13.0);

    let tpidsr = (2.0 * PI) / sr as f32;

    let mut phasor = Phasor::new(sr, 0.0);
    let mut rephasor = RePhasor::new();

    voice2.glot.set_shape(0.5);

    let shape1 = [1.32, 0.44, 0.463, 0.5, 1.44, 2.725, 2.868, 1.606];

    let shape2 = [3.344, 0.44, 0.463, 0.5, 3.154, 3.225, 0.416, 0.463];

    let mut shape: [f32; 8] = [1.0; 8];

    phasor.set_freq(1.0);

    for _ in 0..(sr as f32 * 20.0) as usize {
        let phs = phasor.tick();
        rephasor.set_scale(3.0);
        let rephs = rephasor.tick(phs);

        // slowly morph between two tract shapes
        let shaping = phs;
        for i in 0..8 {
            shape[i] = shaping * shape2[i] + (1.0 - shaping) * shape1[i];
        }

        // apply drm and convert it to raw area functions
        voice1.tract.drm(&shape);
        voice1.glot.set_freq(mtof(63. + 10.0 * phs));

        // slowly morph between two tract shapes
        let shaping = rephs;
        for i in 0..8 {
            shape[i] = shaping * shape2[i] + (1.0 - shaping) * shape1[i];
        }

        // apply drm and convert it to raw area functions
        voice2.tract.drm(&shape);
        voice2.glot.set_freq(mtof(63. + 10.0 * rephs - 4.0));

        let out = voice1.tick() * 0.5 + voice2.tick() * 0.5;
        wav.tick(out);
    }
}
