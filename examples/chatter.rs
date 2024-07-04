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

fn main() {
    let sr = 44100;
    let oversample = 2;
    let tract_len = 8.7;

    let mut wav = MonoWav::new("chatter.wav");

    let mut voice = Voice::new(sr, tract_len, oversample);

    let mut phasor = RandomPhasor::new(sr, 0.0);

    let mut chooser = LinearCongruentialGenerator::new();

    chooser.seed(4444);

    phasor.min_freq = 3.0;
    phasor.max_freq = 10.0;

    let mut drm: [f32; 8] = [0.5; 8];

    let mut jit_freq = Jitter::new(sr);

    jit_freq.seed(4444, 5555);

    jit_freq.range_amplitude(-5., 12.);
    jit_freq.range_rate(3., 10.);

    let mut metro = Metro::new(sr);
    metro.set_rate(1.0);

    let mut tgate = TriggerGate::new(sr);
    tgate.duration = 0.3;

    let tiny_ah = [
        0.77,
        0.855,
        1.435,
        0.728,
        1.067,
        3.217,
        0.671,
        2.892
    ];

    let tiny_ieh = [
        0.6,
        1.081,
        4.,
        3.741,
        0.954,
        0.572,
        0.487,
        1.704
    ];

    let tiny_r4mod1 = [
        0.53,
        1.435,
        0.303,
        3.798,
        2.383,
        0.374,
        2.807,
        0.685

    ];

    let tiny_r4mod2 = [
        0.53,
        1.435,
        0.303,
        0.1,
        2.383,
        0.374,
        2.807,
        0.685

    ];

    let shapes = [
        &tiny_ah,
        &tiny_ieh,
        &tiny_r4mod2,
        &tiny_r4mod1,
    ];

    let mut cur = 0;
    let mut nxt = 1;

    let mut pphs = -1.;

    voice.tract.drm(&tiny_ieh);
    voice.tract.drm(&tiny_ah);
    voice.tract.drm(&tiny_r4mod1);
    voice.tract.drm(&tiny_r4mod2);
    voice.pitch = 63.;


    for _ in 0 .. (sr as f32 * 5.0) as usize {
        let phs = phasor.tick();
        if phs < pphs {
            cur = nxt;
            nxt = (chooser.randf() * shapes.len() as f32) as usize;
            cur %= shapes.len();
            nxt %= shapes.len();
            // cur += 1;
            // nxt += 1;
            // cur %= shapes.len();
            // nxt %= shapes.len();
        }
        let shp_a = shapes[cur];
        let shp_b = shapes[nxt];

        // let frq_phs = freq_phasor.tick();
        // let frq_jit = freq_randi.tick(frq_phs);

        let jf = jit_freq.tick();
        voice.pitch = 63.0 + jf;

        let alpha = gliss_it(phs, 0.8);
        for i in 0..8 {
            drm[i] =
                (1. - alpha) * shp_a[i] +
                alpha * shp_b[i];

        }

        voice.tract.drm(&drm);

        let t = metro.tick();
        let gt = tgate.tick(t);
        let out = voice.tick() * 0.5 * gt;
        wav.tick(out);
        pphs = phs;
    }
}
