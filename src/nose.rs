use crate::Tract;

// TODO de-duplicate
const SPEED_OF_SOUND: f32 = 343.0; /* m/s @ 20C */

pub struct Nose {
    left: Vec<f32>,
    right: Vec<f32>,
    junc_left: Vec<f32>,
    junc_right: Vec<f32>,
    areas: Vec<f32>,
    diams: Vec<f32>,
    reflections: Vec<f32>,
    reflection_left: f32,
    reflection_right: f32,
    reflection_nose: f32,
    length: usize,
    velum: f32,
}

impl Nose {
    pub fn new(sr: usize, length: f32, oversample: u16) -> Self {
        let nose_length =
            (((length * 0.01) / (SPEED_OF_SOUND as f32 / (sr as f32 * oversample as f32)))).floor() + 1.0;

        let nose_length = nose_length as usize;

        let mut ns = Nose {
            areas: vec![0.0; nose_length],
            left: vec![0.0; nose_length],
            right: vec![0.0; nose_length],
            junc_left: vec![0.0; nose_length],
            junc_right: vec![0.0; nose_length],
            reflections: vec![0.0; nose_length],
            diams: vec![0.0; nose_length],
            length: nose_length,
            reflection_left: 0.0,
            reflection_right: 0.0,
            reflection_nose: 0.0,
            velum: 0.0,
        };

        ns.setup_shape();

        ns
    }
    fn setup_shape(&mut self) {
        let diams = &mut self.diams;

        for i in 0 .. self.length {
            let mut d = 2.0 * (i as f32 / self.length as f32);

            if d < 1.0 {
                d = 0.4 + 1.6*d;
            } else {
                d = 0.5 + 1.5*(2.0 - d);
            }

            // silly translation of ternary expression,
            // for debugging purposes
            if d < 1.9 {
                d = d;
            } else {
                d = 1.9;
            }

            diams[i] = d;
        }
        self.calculate_reflections();
    }

    pub fn set_velum(&mut self, velum: f32) {
        self.velum = velum;
    }

    fn calculate_reflections(&mut self) {
        let areas = &mut self.areas;
        let diams = &self.diams;
        let refl = &mut self.reflections;
        for i in 0 .. self.length {
            areas[i] = diams[i]*diams[i];
        }

        for i in 1 .. self.length {
            refl[i] =
                (areas[i - 1] - areas[i]) /
                (areas[i - 1] + areas[i]);
        }
    }


    pub fn calculate_reflections_with_tract(&mut self, tr: &Tract, nose_start: usize)
    {
        self.diams[0] = self.velum;
        self.areas[0] = self.diams[0]*self.diams[0];
        let sum =
            tr.areas[nose_start] +
            tr.areas[nose_start + 1] +
            self.areas[0];
        self.reflection_left = (2.0 * tr.areas[nose_start] - sum) / sum;
        self.reflection_right = (2.0 * tr.areas[nose_start + 1] - sum) / sum;
        self.reflection_nose = (2.0 * self.areas[0] - sum) / sum;
        //dbg!(self.reflection_right, self.velum);
    }

    pub fn tick(&mut self, tr: &mut Tract, nose_start: usize) -> f32 {
        let tr_jl = &mut tr.junc_left;
        let tr_jr = &mut tr.junc_right;

        let tr_l = &tr.left;
        let tr_r = &tr.right;

        let ns_l = &mut self.left;
        let ns_r = &mut self.right;

        let ns_jl = &mut self.junc_left;
        let ns_jr = &mut self.junc_right;

        let r = self.reflection_left;

        if tr_r[nose_start - 1].is_nan() {
            // dbg!(self.samppos);
            // panic!("NAN");
        }

        if ns_l[0].is_nan() {
            // dbg!(self.samppos);
            // panic!("NAN");
        }

        if tr_l[nose_start].is_nan() {
            // dbg!(self.samppos);
            // panic!("NAN");
        }

        tr_jl[nose_start - 1] =
            r*tr_r[nose_start - 1] +
            (1.0 + r)*(ns_l[0] + tr_l[nose_start]);

        if tr_jl[nose_start - 1].is_nan() {
            // dbg!(self.samppos);
            // panic!("NAN");
        }

        let r = self.reflection_right;

        // TODO check this equation, it looks wrong.
        // shouldn't it match junc_left more?
        tr_jr[nose_start] =
            r*tr_l[nose_start] +
            (1.0 + r)*(tr_r[nose_start - 1] + ns_l[0]);

        if tr_jr[nose_start].is_nan() {
            // dbg!(self.samppos);
            // panic!("NAN");
        }

        // 2024-06-08 21:31 this starts blowing up slowly
        if tr_jr[nose_start] > 20.0 {
            //dbg!(tr_jr[nose_start], r, nose_start, self.samppos);
            //panic!("Large number!");
        }

        let i = 11;
        let sum = tr_jr[i - 1] + tr_jl[i];

        if sum.is_finite() == false {
            //dbg!(tr_jr[i - 1]);
            //panic!("INF");
        }

        let r = self.reflection_nose;
        ns_jr[0] =
            r*ns_l[0] +
            (1.0+r)*(tr_l[nose_start]+tr_r[nose_start - 1]);

        if ns_jr[0].is_nan() {
            // dbg!(self.samppos);
            // panic!("NAN");
        }

        ns_jl[self.length - 1] =
            tr.get_lip_reflection() *ns_r[self.length - 1];

        if ns_jl[self.length - 1].is_nan() {
            // dbg!(self.samppos);
            // panic!("NAN");
        }

        for i in 1 .. self.length {
            let w =
                self.reflections[i] *
                (ns_r[i - 1] + ns_l[i]);
            if w.is_nan() {
                // dbg!(self.samppos);
                // panic!("NAN");
            }
            ns_jr[i] = ns_r[i - 1] - w;
            ns_jl[i - 1] = ns_l[i] + w;
        }

        for i in 0 .. self.length {
            ns_l[i] = ns_jl[i];
            ns_r[i] = ns_jr[i];
        }

        self.right[self.length - 1]
    }
}
