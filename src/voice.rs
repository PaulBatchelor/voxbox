use crate::Tract;
use crate::Glot;
use crate::Nose;

pub struct Voice {
    pub tract: Tract,
    pub glottis: Glot,
    pub nose: Nose,
}

impl Voice {
    pub fn new(sr: usize, length_cm: f32, oversample: u16) -> Self {
        let mut v = Voice {
            tract: Tract::new(sr, length_cm, oversample),
            glottis: Glot::new(sr),
            nose: Nose::new(sr, length_cm * 0.63, oversample),
        };

        v.glottis.set_shape(0.576);
        v.glottis.set_aspiration(0.3);
        v.glottis.set_noise_floor(0.287);
        v
    }

    pub fn tick(&mut self) -> f32 {
        let g = self.glottis.tick();
        let t = self.tract.tick(g);
        t
    }
}

