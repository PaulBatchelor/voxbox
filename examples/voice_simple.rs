use voxbox::*;

fn main() {
    let sr = 44100;
    let oversample = 2;
    let tract_len = 17.0;

    let mut wav = MonoWav::new("voice_simple.wav");

    let mut voice = Voice::new(sr, tract_len, oversample);

    let shape1 = [
        1.011, 0.201, 0.487, 0.440,
        1.297, 2.368, 1.059, 2.225
    ];

    voice.tract.drm(&shape1);
    voice.glottis.set_pitch(63.0);

    for _ in 0 .. (sr as f32 * 5.0) as usize {
        let out = voice.tick() * 0.5;
        wav.tick(out);
    }
}
