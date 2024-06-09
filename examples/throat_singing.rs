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
    glot.set_shape(0.676);
    glot.set_aspiration(0.023);
    glot.set_noise_floor(0.01);

    // 2 tract shapes using the Distinct Region Model (DRM)
    // I tuned these by ear

    let _shape1 = [
        0.005, 0.291, 0.077, 0.089,
        1.791, 2.0, 0.148, 0.125,
    ];

    let _shape2 = [
        0.106, 0.487, 3.987, 1.725,
        0.082, 0.225, 0.749, 0.630
    ];

    let _shape2b = [
        0.106, 0.487, 1.654, 3.654,
        0.082, 0.225, 0.749, 0.630
    ];

    let _shape2c = [
        0.106, 0.487, 1.725, 4.0,
        0.082, 0.225, 0.749, 0.0,
    ];

    // let shape2d = [
    //     0.106, 0.487,
    //     //1.725, 0.82,
    //     3.582, 1.559,

    //     0.082, 0.225, 0.749, 0.0,
    // ];

    // let shape2e = [
    //     0.106, 0.487,
    //     2.94, 1.773,

    //     0.082, 0.225, 0.749, 0.0,
    // ];

    // let shape2f = [
    //     0.106, 0.487,
    //     3.059, 2.725,

    //     0.082, 0.225, 0.749, 0.0,
    // ];

    // let shape2g = [
    //     0.106, 0.487,
    //     3.13, 0.916,

    //     0.082, 0.225, 0.749, 0.0,
    // ];

    // Create a shape to hold interpolated blend of
    // two tract shapes
    let mut shape: [f32; 8]= [1.0; 8];
    let mut tract_smoothers: [Smoother; 8] = [Smoother::new(sr); 8];


    let throat_fifth = [
        // 0.082, 0.082, 1.963, 3.678,
        // 0.035, 0.154, 0.059, 0.001
        
        0.082, 0.201, 1.773, 4.0,
        0.487, 0.201, 0.035, 0.201
    ];

    let throat_seventh = [
        // 0.082, 0.082, 1.963, 0.63,
        // 0.035, 0.154, 0.059, 0.001
        0.082, 0.201, 3.106, 0.94,
        0.487, 0.201, 0.035, 0.201
    ];

    let throat_octave = [
        // 0.082, 0.082, 1.963, 0.820,
        // 0.035, 0.154, 0.059, 0.001
        0.082, 0.201, 3.844, 1.654,
        0.487, 0.201, 0.035, 0.201
    ];

    let throat_ninth = [
        //0.082, 0.082, 1.963, 1.249,
        //0.035, 0.154, 0.059, 0.001
        0.082, 0.201, 3.106, 1.940,
        0.487, 0.201, 0.035, 0.201
    ];

    let throat_third = [
        // 0.082, 0.082,
        // //1.963, 1.749, 0.035
        // //4.0, 3.32, 0.035,
        // 3.844, 3.249, 0.487,
        // 0.154, 0.059, 0.001
        0.082, 0.201, 3.106, 2.820,
        0.487, 0.201, 0.035, 0.201
    ];

    let throat_shapes = [
        &throat_seventh,
        &throat_octave,
        &throat_ninth,
        &throat_third,
        &throat_fifth,
    ];

    let seq = [
        4, 4, 4, 3, 2, 4,
        3, 3, 3, 3, 3, 3,
        1, 2, 3, 1, 2, 3,
        0, 0, 0,
        1, 1, 1, 1, 1, 1
    ];

    let dur = 0.3;
    let mut seqpos = 0;
    let mut counter = (sr as f32 * 0.3) as usize;

    let mut which_shape = seq[seqpos];

    for i in 0..8 {
        let s = &mut tract_smoothers[i];
        s.set_smooth(0.02);
        s.snap_to_value(throat_fifth[i]);
    }
    for n in 0..(sr as f32 * 20.0) as usize {
        // set up some LFOs for vibrato, vibrato amount,
        // and amplitude
        //let vibamt = sin(1.0 / 11.0, n, tpidsr);
        //let vibamt = 0.3 + 0.7*vibamt;
        let vibamt = 1.0;
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
            seqpos += 1;
            seqpos %= seq.len();
            counter = (sr as f32 * dur) as usize;
        }

        let shape_to_use = throat_shapes[which_shape];
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
