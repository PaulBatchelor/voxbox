use crate::RePhasor;

enum Behavior {
   Step,
   Linear,
   GlissSmall,
   GlissMedium,
   GlissLarge,
}

pub struct Gesture {
    prev: f32,
    next: f32,
    behavior: Behavior,
    rephasor: RePhasor,
}

impl Gesture {
    pub fn new() -> Self {
        let g = Gesture {
            prev: 0.0,
            next: 0.0,
            behavior: Behavior::GlissMedium,
            rephasor: RePhasor::new(),
        };

        g
    }
}
