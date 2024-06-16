use voxbox::*;

fn db2lin(db: f32) -> f32 {
    (10.0_f32).powf(db / 20.)
}

fn pitch_gestures(chords: &[[i16;4]]) -> Vec<Vec<GestureVertex>>
{
    let mut sop = vec![];
    let mut alt = vec![];
    let mut ten = vec![];
    let mut bas = vec![];
    let mut paths = vec![];

    for chord in chords.iter() {
        bas.push(GestureVertex {
            val: chord[0] as f32,
            num: 1,
            den: 4,
            bhvr: Behavior::GlissMedium
        });

        ten.push(GestureVertex {
            val: chord[1] as f32,
            num: 1,
            den: 4,
            bhvr: Behavior::GlissMedium
        });

        alt.push(GestureVertex {
            val: chord[2] as f32,
            num: 1,
            den: 4,
            bhvr: Behavior::GlissMedium
        });

        sop.push(GestureVertex {
            val: chord[3] as f32,
            num: 1,
            den: 4,
            bhvr: Behavior::GlissMedium
        });
    }

    paths.push(bas);
    paths.push(ten);
    paths.push(alt);
    paths.push(sop);

    paths
}

fn main() {
    let sr = 44100;
    let oversample = 2;
    let tract_cm_tenor = 16.0;
    let tract_cm_bass = 18.3;
    let tract_cm_alto = 14.;
    let tract_cm_soprano = 12.9;
    //let chord = [0, 7, 0, 4];
    //let chord = [0, 9, 2, 7];
    let base_pitch = 63;
    let mut reverb = BigVerb::new(sr);
    let mut clk = Phasor::new(sr, 0.0);
    clk.set_freq(105.0 / 60.0);
    let mut dcblk = DCBlocker::new(sr);

    let chords = [
        [0, 7, 0, 4],
        [-2, 7, 0, 2],
        [0, 7, 4, 5],
        [0, 4, 5, 7],
        [-4, 3, 0, 8],
        [-5, 5, 2, 10],
        [-12, 0, 4, 12],
        [-12, 8, 4, 12],
        [-12, 7, 4, 12],
        [-12, 14, 4, 12],
        [-12, 12, 4, 12],
        [-12, 16, 4, 12],
        [-12, 16, 2, 12],
        [-12, 16, 5, 12],
        [-12, 16, 5, 12],
        [-12, 16, 5, 12],
        [-12, 16, 5, 12],
    ];

    let mut wav = MonoWav::new("vocal_chords.wav");

    let paths = pitch_gestures(&chords);

    let mut gst_sop = LinearGesture::new();
    gst_sop.init(&paths[3]);
    let mut gst_alt = LinearGesture::new();
    gst_alt.init(&paths[2]);
    let mut gst_ten = LinearGesture::new();
    gst_ten.init(&paths[1]);
    let mut gst_bas = LinearGesture::new();
    gst_bas.init(&paths[0]);

    let mut alto = Voice::new(sr, tract_cm_alto, oversample);
    let mut bass = Voice::new(sr, tract_cm_bass, oversample);
    let mut tenor = Voice::new(sr, tract_cm_tenor, oversample);
    let mut soprano = Voice::new(sr, tract_cm_soprano, oversample);

    let shape_ah_alto = [
        // 1.225,
        // 0.225,
        // 0.392,
        // 0.5,
        // 1.13,
        // 2.059,
        // 0.297,
        // 1.392

        0.768,
        0.5,
        0.5,
        0.5,
        1.454,
        3.368,
        3.082,
        2.74
    ];

    let shape_ah_sop = [
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
    //tenor.glottis.set_pitch_ji((base_pitch - 12) as f32, 4);
    tenor.glottis.set_shape(0.4);
    tenor.glottis.srand(12345);
    tenor.vibrato_rate(6.1);
    tenor.vibrato_depth(0.2);

    bass.tract.drm(&shape_ah_bass);
    bass.vibrato_rate(6.0);
    //bass.glottis.set_pitch((base_pitch + chord[0] - 12) as f32);

    //bass.glottis.set_pitch_ji((base_pitch - 12) as f32, 0);
    bass.glottis.set_shape(0.3);
    bass.glottis.srand(54321);
    bass.glottis.set_aspiration(0.01);

    alto.tract.drm(&shape_ah_alto);
    alto.vibrato_rate(6.0);
    alto.glottis.set_shape(0.3);
    alto.glottis.set_aspiration(0.1);
    alto.glottis.srand(330303);

    soprano.tract.drm(&shape_ah_sop);
    soprano.glottis.set_shape(0.5);
    soprano.glottis.set_aspiration(0.1);
    soprano.glottis.srand(111111);
    soprano.vibrato_rate(6.5);
    soprano.vibrato_depth(0.1);

    reverb.size = 0.95;

    let mut hp1 = ButterworthHighPass::new(sr);
    hp1.set_freq(600.);
    let mut hp2 = ButterworthHighPass::new(sr);
    hp1.set_freq(300.);

    for _ in 0 .. (sr as f32 * 37.0) as usize {
        let phs = clk.tick();

        let pitch = gst_sop.tick(phs);
        soprano.pitch = base_pitch as f32 + pitch;

        let pitch = gst_alt.tick(phs);
        alto.pitch = base_pitch as f32 + pitch;

        let pitch = gst_ten.tick(phs);
        tenor.pitch = (base_pitch - 12) as f32 + pitch;

        let pitch = gst_bas.tick(phs);
        bass.pitch = (base_pitch - 12) as f32 + pitch;

        let b = bass.tick() * db2lin(3.);
        let t = tenor.tick() * db2lin(-1.);
        let a = alto.tick() * db2lin(0.);

        let s = soprano.tick() * db2lin(5.);

        let sa = hp1.tick(a + s);
        let sum = (sa + b + t) * db2lin(-15.);
        //let sum = (b) * db2lin(-13.);
        //let sum = (a) * db2lin(-13.);
        let rvbin = hp2.tick(sum);
        let (rvb, _) = reverb.tick(rvbin, rvbin);
        let rvb = dcblk.tick(rvb);
        //let out = sum;
        let out = sum + rvb * db2lin(-14.);
        let out = out * db2lin(-1.);
        wav.tick(out);
    }
}
