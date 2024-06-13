use voxbox::*;

fn main() {
    let sr = 44100;
    let oversample = 2;
    let tract_cm_tenor = 16.0;
    let tract_cm_bass = 18.0;
    let tract_cm_alto = 15.0;
    let tract_cm_soprano = 12.0;
    let chord = [0, 7, 0, 4]; 
    let base_pitch = 63;

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
        // 1.32,
        // 0.44,
        // 0.463,
        // 0.5,
        // 1.44,
        // 2.725,
        // 2.868,
        // 1.606
       1.059,
        0.273,
        0.32,
        0.392,
        1.535,
        2.368,
        0.511,
        2.963
    ];

    let shape_ah_sop = [
        //1.059,
        //0.273,
        //0.32,
        //0.392,
        //1.535,
        //2.368,
        //0.511,
        //2.963
        0.106,
        0.011,
        0.082,
        0.178,
        2.13,
        3.701,
        3.106,
        1.63
    ];

    let shape_ah_tenor = [
        0.225,
        0.059,
        0.059,
        0.082,
        0.701,
        3.701,
        1.725,
        1.082
    ];

    let shape_ah_bass = [
        0.225,
        0.059,
        0.059,
        0.082,
        0.701,
        3.868,
        0.32,
        0.963
    ];

    tenor.tract.drm(&shape_ah_tenor);
    //tenor.glottis.set_pitch((base_pitch + chord[1] - 12) as f32);
    tenor.pitch = (base_pitch + chord[1] - 12) as f32;
    //tenor.glottis.set_pitch_ji((base_pitch - 12) as f32, 4);
    tenor.glottis.set_shape(0.6);
    tenor.glottis.srand(12345);

    bass.tract.drm(&shape_ah_bass);
    //bass.glottis.set_pitch((base_pitch + chord[0] - 12) as f32);
    bass.pitch = (base_pitch + chord[0] - 12) as f32;

    //bass.glottis.set_pitch_ji((base_pitch - 12) as f32, 0);
    bass.glottis.set_shape(0.7);
    bass.glottis.srand(54321);

    alto.tract.drm(&shape1);
    //alto.glottis.set_pitch_ji(base_pitch as f32, 0);
    //alto.glottis.set_pitch((base_pitch + chord[2]) as f32);
    alto.pitch = (base_pitch + chord[2]) as f32;
    alto.glottis.set_shape(0.4);
    alto.glottis.set_aspiration(0.1);
    alto.glottis.srand(333333);

    soprano.tract.drm(&shape_ah_sop);
    //soprano.glottis.set_pitch((base_pitch + chord[3]) as f32);
    soprano.pitch = (base_pitch + chord[3]) as f32;
    //soprano.glottis.set_pitch_ji(base_pitch as f32, 2);
    soprano.glottis.set_shape(0.6);
    soprano.glottis.set_aspiration(0.1);
    soprano.glottis.srand(111111);

    for _ in 0 .. (sr as f32 * 5.0) as usize {
        let b = bass.tick();
        let t = tenor.tick();
        let a = alto.tick();
        let s = soprano.tick();
        let solo_gain = 0.6;

        let sum = (s + a + t + b) * 0.2;
        wav.tick(sum);
        wav_bass.tick(b * solo_gain);
        wav_tenor.tick(t * solo_gain);
        wav_alto.tick(a * solo_gain);
        wav_soprano.tick(s * solo_gain);
    }
}
