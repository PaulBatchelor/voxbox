use voxbox::*;

fn vtx(val: f32, dur: &[u32], bhvr: Behavior) -> GestureVertex<f32> {
    GestureVertex {
        val: val,
        num: dur[1],
        den: dur[0],
        bhvr: bhvr,
    }
}

fn main() {
    let sr = 44100;
    let oversample = 2;
    let tract_len = 13.0;

    let mut wav = MonoWav::new("gesture_builder.wav");
    let mut voice = Voice::new(sr, tract_len, oversample);
    let gm = Behavior::GlissMedium;
    let gl = Behavior::GlissLarge;
    let gh = Behavior::GlissHuge;
    let base = 66.0;
    let e = &[1, 2];
    let s = &[1, 4];
    let edot = &[3, 4];
    let mut reverb = BigVerb::new(sr);

    voice.glottis.set_aspiration(0.3);
    let nt = |nn: u16, dur| -> GestureVertex<f32> { vtx(base + nn as f32, dur, gm) };

    let ntb = |nn: u16, dur, bvr| -> GestureVertex<f32> { vtx(base + nn as f32, dur, bvr) };

    let gpath = vec![
        nt(0, e),
        nt(2, e),
        nt(3, e),
        nt(7, e),
        nt(0, e),
        nt(2, s),
        nt(3, edot),
        nt(7, e),
        ntb(0, e, gh),
        nt(7, e),
        nt(5, e),
        nt(3, e),
        nt(5, e),
        nt(3, s),
        nt(5, edot),
        ntb(3, e, gl),
    ];

    let mut clk = Phasor::new(sr, 0.0);

    clk.set_freq(98.0 / 60.0);

    let mut gst = LinearGestureBuilder::new();

    for vtx in gpath.into_iter() {
        gst.append(vtx);
    }

    gst.done();

    let shape1 = [1.011, 0.201, 0.487, 0.440, 1.297, 2.368, 1.059, 2.225];

    voice.tract.drm(&shape1);
    reverb.size = 0.75;

    for _ in 0..(sr as f32 * 10.0) as usize {
        let c = clk.tick();
        let pitch = gst.tick(c);
        voice.pitch = pitch;
        let out = voice.tick() * 0.5;
        let (rvb, _) = reverb.tick(out, out);
        wav.tick(out + rvb * 0.08);
    }
}
