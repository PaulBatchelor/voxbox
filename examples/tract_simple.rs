use voxbox::MonoWav;
use voxbox::Glot;
use voxbox::Tract;
use std::f32::consts::PI;

pub fn mtof(nn: f32) -> f32 {
    let freq = (2.0_f32).powf((nn - 69.0) / 12.0) * 440.0;
    freq
}

pub fn sin(frq: f32, n: usize, tpidsr: f32) -> f32 {
    let lfo = (frq * n as f32 * tpidsr).sin();
    let lfo = (lfo + 1.0) * 0.5;
    lfo
}

fn main() {
    let sr = 44100;
    let mut wav = MonoWav::new("tract_simple.wav");
    let mut glot = Glot::new(sr);
    let mut tract = Tract::new(sr, 13.0, 1);

    let tpidsr = (2.0 * PI) / sr as f32;

    glot.set_shape(0.3);
    glot.set_aspiration(0.9);
    glot.set_noise_floor(0.05);

    let shape1 = [
        2.0, 3.0, 9.0, 2.0,
        1.0, 1.0, 1.0, 1.0,
    ];

    let shape2 = [
        1.0, 1.0, 1.0, 1.0,
        2.0, 3.0, 9.0, 2.0,
    ];

    let mut shape: [f32; 8]= [1.0; 8];

    for n in 0..(sr as f32 * 20.0) as usize {
        let s = glot.tick() * 0.7;

        let vibamt = 1.0;

        let shaping = sin(1.0 / 6.0, n , tpidsr);
        let amp = sin(1.0 / 7.0, n, tpidsr);
        let vib = ((5.8 + 0.2*amp) * n as f32 * tpidsr).sin();
        let vib = (vib + 1.0) * 0.5;

        for i in 0 .. 8 {
            shape[i] = shaping * shape1[i] + (1.0 - shaping)*shape2[i];
        }

        tract.drm(&shape);

        glot.set_freq(mtof(65. + 0.4 * vib * vibamt * amp));

        let t = tract.tick(s) * amp;
        wav.tick(t * 0.5);
    }
}
