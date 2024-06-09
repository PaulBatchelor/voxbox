use voxbox::MonoWav;
use voxbox::Glot;
use voxbox::Tract;
use voxbox::Nose;
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

    let tractlen = 20.0;
    let oversample = 1;

    let mut tract = Tract::new(sr, tractlen, oversample);

    let mut nose = Nose::new(sr, tractlen * 0.63, oversample);

    // 2 pi / samplerate constant for ad-hoc sine oscillators
    let tpidsr = (2.0 * PI) / sr as f32;

    // some glottal parameter settings
    glot.set_shape(0.619);
    glot.set_aspiration(0.023);
    glot.set_noise_floor(0.01);

    // 2 tract shapes using the Distinct Region Model (DRM)
    // I tuned these by ear

    let shape1 = [
        0.005, 0.291, 0.077, 0.089,
        1.791, 2.0, 0.148, 0.125,
    ];

    let shape2 = [
        0.106, 0.487, 3.987, 1.725,
        0.082, 0.225, 0.749, 0.630
    ];

    let shape2b = [
        0.106, 0.487, 1.654, 3.654,
        0.082, 0.225, 0.749, 0.630
    ];

    let shape2c = [
        0.106, 0.487, 1.725, 4.0,
        0.082, 0.225, 0.749, 0.0,
    ];

    let shape2d = [
        0.106, 0.487,
        //1.725, 0.82,
        3.582, 1.559,

        0.082, 0.225, 0.749, 0.0,
    ];

    // Create a shape to hold interpolated blend of
    // two tract shapes
    let mut shape: [f32; 8]= [1.0; 8];
    let mut which_shape = 0;

    for n in 0..(sr as f32 * 10.0) as usize {
        // set up some LFOs for vibrato, vibrato amount,
        // and amplitude
        //let vibamt = sin(1.0 / 11.0, n, tpidsr);
        //let vibamt = 0.3 + 0.7*vibamt;
        let vibamt = 1.0;
        let amp = sin(1.0 / 7.0, n, tpidsr);
        let amp = 1.0;
        let vib = ((5.3 + 0.1*amp) * n as f32 * tpidsr).sin();
        let vib = (vib + 1.0) * 0.5;

        // slowly morph between two tract shapes
        let shaping = sin(1.0 / 3.0, n , tpidsr);
        for i in 0 .. 8 {
            shape[i] = shaping * shape1[i] + (1.0 - shaping)*shape2[i];
        }

        if (n % sr) == 0 {
            if which_shape == 0 {
                which_shape = 1;
            } else {
                which_shape = 0;

            }
        }

        // apply drm and convert it to raw area functions
        if which_shape == 0 {
            tract.drm(&shape2c);
        } else {
            tract.drm(&shape2d);
        }

        // set glottal source frequency
        glot.set_freq(mtof(62. + 0.3 * vib * vibamt * amp - 12.0));

        nose.set_velum(4.0);
        // processing and write WAV
        let s = glot.tick() * 0.7;
        //let t = tract.tick(s);
        let t = tract.tick_with_nose(&mut nose, s);
        wav.tick(t * 0.5);
    }
}
