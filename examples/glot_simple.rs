use voxbox::MonoWav;
use voxbox::Glot;

pub fn mtof(nn: f32) -> f32 {
    let freq = (2.0_f32).powf((nn - 69.0) / 12.0) * 440.0;
    freq
}

fn main() {
    let sr = 44100;
    let mut wav = MonoWav::new("glot_simple.wav");
    let mut glot = Glot::new(sr);

    glot.set_freq(mtof(60.));

    for _ in 0..(sr as f32 * 2.0) as usize {
        let s = glot.tick() * 0.7;
        wav.tick(s);
    }
}
