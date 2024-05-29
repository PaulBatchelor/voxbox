use voxbox;

fn main() {
    println!("Hello voxbox!");

    let mut wav = voxbox::MonoWav::new("test.wav");

    for _ in 0..44100 {
        wav.tick(0.);
    }
}
