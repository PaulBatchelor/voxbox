use crate::Tract;
use crate::Glot;
use crate::Nose;
use crate::Phasor;
use std::f32::consts::PI;

pub struct Voice {
    pub tract: Tract,
    pub glottis: Glot,
    pub nose: Nose,
    pub pitch: f32,
    phasor: Phasor,
}

impl Voice {
    pub fn new(sr: usize, length_cm: f32, oversample: u16) -> Self {
        let mut v = Voice {
            tract: Tract::new(sr, length_cm, oversample),
            glottis: Glot::new(sr),
            nose: Nose::new(sr, length_cm * 0.63, oversample),
            phasor: Phasor::new(sr, 0.0),
            pitch: 60.0,
        };

        v.glottis.set_shape(0.476);
        v.glottis.set_aspiration(0.1);
        v.glottis.set_noise_floor(0.287);
        v.phasor.set_freq(6.0);
        v
    }

    pub fn vibrato_rate(&mut self, rate: f32) {
        self.phasor.set_freq(rate);
    }

    pub fn tick(&mut self) -> f32 {
        let phs = self.phasor.tick();
        let vib = (phs * 2.0*PI).sin() * 0.03;
        self.glottis.set_pitch(self.pitch + vib);
        let g = self.glottis.tick();
        let t = self.tract.tick(g);
        t
    }
}

