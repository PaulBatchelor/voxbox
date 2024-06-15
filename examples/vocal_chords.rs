use voxbox::*;

fn db2lin(db: f32) -> f32 {
    (10.0_f32).powf(db / 20.)
}

fn main() {
    let sr = 44100;
    let oversample = 2;
    let tract_cm_tenor = 16.0;
    let tract_cm_bass = 18.3;
    let tract_cm_alto = 14.3;
    let tract_cm_soprano = 12.9;
    let chord = [0, 7, 0, 4]; 
    //let chord = [0, 9, 2, 7]; 
    let base_pitch = 63;
    let mut reverb = BigVerb::new(sr);

    let mut wav = MonoWav::new("vocal_chords.wav");

    let mut alto = Voice::new(sr, tract_cm_alto, oversample);
    let mut bass = Voice::new(sr, tract_cm_bass, oversample);
    let mut tenor = Voice::new(sr, tract_cm_tenor, oversample);
    let mut soprano = Voice::new(sr, tract_cm_soprano, oversample);

    let shape_ah_alto = [
        // 1.32,
        // 0.44,
        // 0.463,
        // 0.5,
        // 1.44,
        // 2.725,
        // 2.868,
        // 1.606
       // 1.059,
       //  0.273,
       //  0.32,
       //  0.392,
       //  1.535,
       //  2.368,
       //  0.511,
       //  2.963

        // 0.225,
        // 0.154,
        // 0.392,
        // 0.5,
        // 0.5,
        // 2.13,
        // 3.511,
        // 2.059

        1.225,
        0.225,
        0.392,
        0.5,
        1.13,
        2.059,
        0.297,
        1.392
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
        
        // 0.106,
        // 0.011,
        // 0.082,
        // 0.178,
        // 2.13,
        // 3.701,
        // 3.106,
        // 1.63,


        1.773,
        0.225,
        0.392,
        0.5,
        1.868,
        1.987,
        0.392,
        3.249
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
        // 0.225,
        // 0.059,
        // 0.059,
        // 0.082,
        // 0.701,
        // 3.868,
        // 0.32,
        // 0.963

        0.225,
        0.63,
        0.844,
        0.5,
        0.5,
        3.701,
        0.82,
        2.106
    ];

    tenor.tract.drm(&shape_ah_tenor);
    //tenor.glottis.set_pitch((base_pitch + chord[1] - 12) as f32);
    tenor.pitch = (base_pitch + chord[1] - 12) as f32;
    //tenor.glottis.set_pitch_ji((base_pitch - 12) as f32, 4);
    tenor.glottis.set_shape(0.6);
    tenor.glottis.srand(12345);

    bass.tract.drm(&shape_ah_bass);
    //bass.glottis.set_pitch((base_pitch + chord[0] - 12) as f32);
    bass.pitch = (base_pitch + chord[0] - 24) as f32;

    bass.glottis.set_pitch_ji((base_pitch - 12) as f32, 0);
    //bass.glottis.set_shape(0.4);
    bass.glottis.srand(54321);

    alto.tract.drm(&shape_ah_alto);
    alto.pitch = (base_pitch + chord[2]) as f32;
    alto.glottis.set_shape(0.4);
    alto.glottis.set_aspiration(0.1);
    alto.glottis.srand(333333);

    soprano.tract.drm(&shape_ah_sop);
    soprano.pitch = (base_pitch + chord[3]) as f32;
    soprano.glottis.set_shape(0.4);
    soprano.glottis.set_aspiration(0.1);
    soprano.glottis.srand(111111);

    reverb.size = -0.97;
    for _ in 0 .. (sr as f32 * 10.0) as usize {
        let b = bass.tick() * db2lin(0.);
        let t = tenor.tick() * db2lin(-4.);
        let a = alto.tick() * db2lin(-4.);
        let s = soprano.tick() * db2lin(1.);
        let solo_gain = 0.6;

        let sum = (s + a + t + b) * 0.2;
        let (rvb, _) = reverb.tick(sum, sum);
        wav.tick(sum*db2lin(-1.0) + rvb * db2lin(-18.));
    }
}
