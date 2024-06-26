use crate::RePhasor;
//use std::option;

#[derive(Copy, Clone)]
pub enum Behavior {
   Step,
   Linear,
   GlissTiny,
   GlissSmall,
   GlissMedium,
   GlissLarge,
   GlissHuge,
}

pub struct Gesture<T> {
    prev: T,
    next: T,
    ratemul: T,
    behavior: Behavior,
    rephasor: RePhasor,
    lphs: T,
    next_behavior: Behavior,
}

pub struct LinearGesture<'a> {
    gest: Gesture<f32>,
    //path: &'a [GestureVertex],
    path: Option<&'a Vec<GestureVertex<f32>>>,
    pos: usize,
}

#[derive(Copy, Clone)]
pub struct GestureVertex<T> {
    pub val: T,
    pub num: u32,
    pub den: u32,
    pub bhvr: Behavior,
}

pub trait SignalGenerator {
    fn next_vertex(&mut self) -> GestureVertex<f32>;
    fn compute_rephasor(&mut self, clk: f32) -> f32;
    fn interpolate(&mut self, phs: f32) -> f32;
    fn new_period(&mut self, phs: f32) -> bool;
    fn tick(&mut self, clk: f32) -> f32 {
        let phs = self.compute_rephasor(clk);

        if self.new_period(phs) {
            let vtx = self.next_vertex();
            self.update(&vtx);
        }
        self.interpolate(phs)
    }
    fn update(&mut self, vtx: &GestureVertex<f32>);
}


impl SignalGenerator for Gesture<f32> {
    fn new_period(&mut self, phs: f32) -> bool {
        self.lphs > phs
    }

    fn next_vertex(&mut self) -> GestureVertex<f32> {
        GestureVertex {
            val: 0.0, num: 1, den: 1, bhvr: Behavior::Linear
        }
    }

    fn compute_rephasor(&mut self, clk: f32) -> f32
    {
        self.rephasor.tick(clk)
    }

    fn interpolate(&mut self, phs: f32) -> f32 {
        let a = apply_behavior(phs, &self.behavior);

        let out =
            (1.0 - a)*self.prev +
            a * self.next;

        self.lphs = phs;

        out
    }

    fn update(&mut self, vtx: &GestureVertex<f32>) {
        // Set the previous rate multiplier
        // because we want this relationship: A -> A_rm (A_bhvr) -> B
        self.rephasor.set_scale(self.ratemul);

        // with that set, we can cache the upcoming RM
        self.ratemul = vtx.num as f32 / vtx.den as f32;
        self.prev = self.next;
        self.next = vtx.val;
        self.behavior = self.next_behavior;
        self.next_behavior = vtx.bhvr;
    }

}

impl Gesture<f32> {
    pub fn new() -> Self {
        let g = Gesture {
            prev: 0.0,
            next: 0.0,
            ratemul: 1.0,
            behavior: Behavior::GlissMedium,
            next_behavior: Behavior::GlissMedium,
            rephasor: RePhasor::new(),
            // triggers update on init
            lphs: 1.0,
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
            gliss_it(phs, 0.85)
        },
        Behavior::GlissLarge => {
            gliss_it(phs, 0.5)
        },
        Behavior::GlissHuge => {
            gliss_it(phs, 0.1)
        },
        Behavior::GlissTiny => {
            gliss_it(phs, 0.9)
        },
    };

    out
}

impl<'a> LinearGesture<'a> {
    //pub fn new(path: &'a Vec<GestureVertex>) -> Self {
    //    let mut lg = LinearGesture {
    //        gest: Gesture::new(),
    //        path: path,
    //        pos: 0,
    //    };

    //    lg.init();

    //    lg
    //}
    pub fn new() -> Self {
        let lg = LinearGesture {
            gest: Gesture::new(),
            path: None,
            pos: 0,
        };

        lg
    }

    pub fn init(&mut self, path: &'a Vec<GestureVertex<f32>>) {
        self.path = Some(path);
        // get vertex, now next vertex is on deck
        let a = self.next_vertex();
        // this is called before the first tick
        // important values are ratemul and next
        self.update(&a);

        // tick will be called on the first call,
        // which will set this next value to be prev
        // the rate multiplier here will also be held onto
    }
}

impl SignalGenerator for LinearGesture<'_> {
    fn next_vertex(&mut self) -> GestureVertex<f32> {
        let next = match self.path {
            Some(x) => {
                let nxt = x[self.pos];
                self.pos += 1;
                if self.pos >= x.len() {
                    self.pos = 0;
                }
                nxt
            },

            None => {
                GestureVertex {
                    val: 0.0, num: 1, den: 1, bhvr: Behavior::Linear
                }
            }
        };
        next
    }

    fn compute_rephasor(&mut self, clk: f32) -> f32 {
        self.gest.compute_rephasor(clk)
    }

    fn interpolate(&mut self, phs: f32) -> f32 {
        self.gest.interpolate(phs)
    }

    fn new_period(&mut self, phs: f32) -> bool {
        self.gest.new_period(phs)
    }

    fn update(&mut self, vtx: &GestureVertex<f32>) {
        self.gest.update(vtx);
    }

}
