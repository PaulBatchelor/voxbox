use voxbox::*;

fn main() {
    let sr = 44100;
    let oversample = 2;
    let tract_len = 8.7;

    let mut wav = MonoWav::new("chatter.wav");

    let mut voice = Voice::new(sr, tract_len, oversample);

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

    voice.tract.drm(&tiny_ieh);
    voice.tract.drm(&tiny_ah);
    voice.tract.drm(&tiny_r4mod1);
    voice.tract.drm(&tiny_r4mod2);
    voice.glottis.set_pitch(63.0);

    for _ in 0 .. (sr as f32 * 5.0) as usize {
        let out = voice.tick() * 0.5;
        wav.tick(out);
    }
}
