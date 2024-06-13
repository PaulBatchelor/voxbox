use crate::RePhasor;

pub enum Behavior {
   Step,
   Linear,
   GlissSmall,
   GlissMedium,
   GlissLarge,
}

pub struct Gesture {
    prev: f32,
    next: f32,
    ratemul: f32,
    behavior: Behavior,
    rephasor: RePhasor,
    lphs: f32,
}

pub struct GestureVertex {
    val: f32,
    num: f32,
    den: f32,
    bhvr: Behavior,
}

pub trait SignalGenerator {
    fn next_vertex(&self) -> GestureVertex;
    fn tick(&mut self, clk: f32) -> f32;
}

impl SignalGenerator for Gesture {
    fn next_vertex(&self) -> GestureVertex {
        GestureVertex {
            val: 0.0, num: 1.0, den: 1.0, bhvr: Behavior::Linear
        }
    }

    fn tick(&mut self, clk: f32) -> f32 {
        self.rephasor.set_scale(self.ratemul);
        let phs = self.rephasor.tick(clk);

        if self.lphs < phs {
            // TODO: update prev/next and ratemul
            let vtx = self.next_vertex();
        }

        let a = apply_behavior(phs, &self.behavior);

        let out =
            (1.0 - a)*self.prev +
            a * self.next;

        self.lphs = phs;

        out
    }
}

impl Gesture {
    pub fn new() -> Self {
        let g = Gesture {
            prev: 0.0,
            next: 0.0,
            ratemul: 1.0,
            behavior: Behavior::GlissMedium,
            rephasor: RePhasor::new(),
            lphs: -1.0,
        };

        g
    }


}

fn gliss_it(phs: f32, glisspos: f32) -> f32 {
    let mut a;
    if phs < glisspos {
        a = 0.0;
    } else {
        a = phs - glisspos;
        if a < 0.0 {
            a = 0.0;
        }
        a /= 1.0 - glisspos;
        a = a * a * a;
    }
    a
}

fn apply_behavior(phs: f32, bhvr: &Behavior) -> f32 {
    let out = match bhvr {
        Behavior::Step => {
            0.0
        },
        Behavior::Linear => {
            phs
        },
        Behavior::GlissMedium => {
            gliss_it(phs, 0.75) 
        },
        Behavior::GlissSmall => {
            gliss_it(phs, 0.9) 
        },
        Behavior::GlissLarge => {
            gliss_it(phs, 0.5) 
        },
    };

    out
}

