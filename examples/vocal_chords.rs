use voxbox::*;

fn main() {
    let sr = 44100;
    let oversample = 2;
    let tract_cm_tenor = 18.0;
    let tract_cm_bass = 20.0;
    let tract_cm_alto = 14.0;
    let tract_cm_soprano = 12.0;
    let chord = [0, 7, 0, 4]; 
    let base_pitch = 67;

    let mut wav = MonoWav::new("vocal_chords.wav");
    let mut wav_bass = MonoWav::new("vocal_chords_bass.wav");
    let mut wav_tenor = MonoWav::new("vocal_chords_tenor.wav");
    let mut wav_alto = MonoWav::new("vocal_chords_alto.wav");
    let mut wav_soprano = MonoWav::new("vocal_chords_soprano.wav");

    let mut alto = Voice::new(sr, tract_cm_alto, oversample);
    let mut bass = Voice::new(sr, tract_cm_bass, oversample);
    let mut tenor = Voice::new(sr, tract_cm_tenor, oversample);
    let mut soprano = Voice::new(sr, tract_cm_soprano, oversample);

    let shape1 = [
        1.32,
        0.44,
        0.463,
        0.5,
        1.44,
        2.725,
        2.868,
        1.606
    ];

    tenor.tract.drm(&shape1);
    tenor.glottis.set_pitch((base_pitch + chord[1] - 12) as f32);

    bass.tract.drm(&shape1);
    bass.glottis.set_pitch((base_pitch + chord[0] - 12) as f32);

    alto.tract.drm(&shape1);
    alto.glottis.set_pitch((base_pitch + chord[2]) as f32);

    soprano.tract.drm(&shape1);
    soprano.glottis.set_pitch((base_pitch + chord[3]) as f32);
    soprano.glottis.set_shape(0.7);

    for _ in 0 .. (sr as f32 * 5.0) as usize {
        let b = bass.tick();
        let t = tenor.tick();
        let a = alto.tick();
        let s = soprano.tick();
        let solo_gain = 0.6;

        let sum = (s + a + t + b) * 0.1;
        wav.tick(sum);
        wav_bass.tick(b * solo_gain);
        wav_tenor.tick(t * solo_gain);
        wav_alto.tick(a * solo_gain);
        wav_soprano.tick(s * solo_gain);
    }
}
