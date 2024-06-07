use voxbox::MonoWav;
use voxbox::Glot;
use voxbox::Tract;
use std::f32::consts::PI;


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
    let mut wav = MonoWav::new("velum_simple.wav");

    // this is a source-filter model. glot is the source,
    // tract is the filter.
    let mut glot = Glot::new(sr);

    let mut tract = Tract::new(sr, 20.0, 1);

    // 2 pi / samplerate constant for ad-hoc sine oscillators
    let tpidsr = (2.0 * PI) / sr as f32;

    // some glottal parameter settings
    glot.set_shape(0.4);
    glot.set_aspiration(0.1);
    glot.set_noise_floor(0.01);

    // 2 tract shapes using the Distinct Region Model (DRM)
    // I tuned these by ear

    let shape1 = [
        0.437, 0.074, 0.205, 0.098,
        0.217, 0.044, 0.207, 0.342
    ];

    let shape2 = [
        0.437, 0.074, 0.205, 0.098,
        0.848, 0.044, 0.207, 0.342
    ];

    // Create a shape to hold interpolated blend of
    // two tract shapes
    let mut shape: [f32; 8]= [1.0; 8];

    for n in 0..(sr as f32 * 20.0) as usize {
        // set up some LFOs for vibrato, vibrato amount,
        // and amplitude
        let vibamt = sin(1.0 / 11.0, n, tpidsr);
        let vibamt = 0.3 + 0.7*vibamt;
        let amp = sin(1.0 / 7.0, n, tpidsr);
        let vib = ((5.3 + 0.1*amp) * n as f32 * tpidsr).sin();
        let vib = (vib + 1.0) * 0.5;

        // slowly morph between two tract shapes
        let shaping = sin(1.0 / 6.0, n , tpidsr);
        for i in 0 .. 8 {
            shape[i] = shaping * shape1[i] + (1.0 - shaping)*shape2[i];
        }

        // apply drm and convert it to raw area functions
        tract.drm(&shape);

        // set glottal source frequency
        glot.set_freq(mtof(65. + 0.3 * vib * vibamt * amp - 12.0 - 10.0 -5.0));

        // processing and write WAV
        let s = glot.tick() * 0.7;
        let t = tract.tick(s);
        wav.tick(t * 0.5);
    }
}
