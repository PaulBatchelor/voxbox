use voxbox::MonoWav;
use voxbox::Glot;
use voxbox::Tract;
use voxbox::Nose;
use voxbox::Smoother;
use voxbox::BigVerb;
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
    let mut wav = MonoWav::new("throat_singing.wav");

    // this is a source-filter model. glot is the source,
    // tract is the filter.
    let mut glot = Glot::new(sr);

    let tractlen = 20.0;
    let oversample = 1;

    let mut tract = Tract::new(sr, tractlen, oversample);

    let mut nose = Nose::new(sr, tractlen * 0.63, oversample);

    let mut reverb = BigVerb::new(sr);
    reverb.init();
    reverb.size = 0.97;

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

    let shape2e = [
        0.106, 0.487,
        2.94, 1.773,

        0.082, 0.225, 0.749, 0.0,
    ];

    let shape2f = [
        0.106, 0.487,
        3.059, 2.725,

        0.082, 0.225, 0.749, 0.0,
    ];

    let shape2g = [
        0.106, 0.487,
        3.13, 0.916,

        0.082, 0.225, 0.749, 0.0,
    ];

    // Create a shape to hold interpolated blend of
    // two tract shapes
    let mut shape: [f32; 8]= [1.0; 8];
    let mut which_shape = 0;

    let mut tract_smoothers: [Smoother; 8] = [Smoother::new(sr); 8];

    for i in 0..8 {
        let s = &mut tract_smoothers[i];
        s.set_smooth(0.02);
        s.snap_to_value(shape2d[i]);
    }

    let mut shape_to_use = &shape2d;

    let throat_shapes = [
        &shape2g,
        &shape2d,
        &shape2e,
        &shape2f,
        &shape2c,
    ];

    let seq = [
        4, 4, 4, 3, 2, 4,
        3, 3, 3, 3, 3, 3,
        1, 2, 3, 1, 2, 3,
        0, 0, 0,
        1, 1, 1, 1, 1, 1];

    let dur = 0.3;
    let mut seqpos = 0;
    let mut counter = (sr as f32 * 0.3) as usize;

    which_shape = seq[seqpos];
    for n in 0..(sr as f32 * 20.0) as usize {
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
        // let shaping = sin(1.0 / 3.0, n , tpidsr);
        // for i in 0 .. 8 {
        //     shape[i] = shaping * shape1[i] + (1.0 - shaping)*shape2[i];
        // }

        counter -= 1;
        if counter == 0 {
            //which_shape += 1;
            //which_shape %= throat_shapes.len();
            which_shape = seq[seqpos];
            seqpos += 1;;
            seqpos %= seq.len();
            counter = (sr as f32 * dur) as usize;
        }

        // apply drm and convert it to raw area functions
        //if which_shape == 0 {
        //    //tract.drm(&shape2c);
        //    shape_to_use = &shape2c;
        //} else {
        //    shape_to_use = &shape2d;
        //}

        shape_to_use = throat_shapes[which_shape];
        for i in 0..8 {
            shape[i] = tract_smoothers[i].tick(shape_to_use[i]);
        }

        tract.drm(&shape);

        // set glottal source frequency
        glot.set_freq(mtof(62. + 0.3 * vib * vibamt * amp - 12.0));

        nose.set_velum(4.0);
        // processing and write WAV
        let s = glot.tick() * 0.7;
        //let t = tract.tick(s);
        let t = tract.tick_with_nose(&mut nose, s);
        let t = t * 0.5;

        let (rvb_l, _) = reverb.tick(t, t);

        let out = t*0.8 + rvb_l * 0.2;
        wav.tick(out);

    }
}
