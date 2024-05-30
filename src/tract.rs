use std::f32::consts::PI;

pub struct Tract {
    // TODO: how to use dynbox instead?
    // left/right delay waveguides
    left: Vec<f32>,
    right: Vec<f32>,
    junc_left: Vec<f32>,
    junc_right: Vec<f32>,

    tractlen: u16,

    areas: Vec<f32>,
    reflections: Vec<f32>,
    diams: Vec<f32>,

    // anti-aliasing (aliasing supression)
    hp: f32,
    c1: f32,
    c2: f32,
    yt1: f32,
    prvhp: f32,
    tpidsr: f32,
    oversample: u16,
}

impl Tract {
    pub fn new(sr: usize, size: usize, oversample: u16) -> Self {
        let mut tr = Tract {
            areas: vec![0.0; size],
            left: vec![0.0; size],
            right: vec![0.0; size],
            junc_left: vec![0.0; size],
            junc_right: vec![0.0; size],
            diams: vec![0.0; size],
            reflections: vec![0.0; size],
            c1: 0.0,
            c2: 0.0,
            hp: 0.0,
            yt1: 0.0,
            oversample: oversample,
            prvhp: 0.0,
            tpidsr: 2.0 * PI / (sr as f32 * oversample as f32),
            tractlen: size as u16,
        };

        tr.setup_antialiasing_filter(sr);

        tr
    }

    fn setup_antialiasing_filter(&mut self, sr: usize) {
        // a little less than nyquist, darker is better
        self.hp = sr as f32 * 0.4;
        self.prvhp = self.hp;
        let b = 2.0 - (self.prvhp * self.tpidsr).cos();
        self.c2 = b - (b*b - 1.0).sqrt();
        self.c1 = 1.0 - self.c2;
        self.yt1 = 0.0;
    }

    fn compute_areas_from_diams(&mut self) {
        let mut a = &mut self.areas;
        let mut d = &mut self.diams;

        for i in 0 .. self.tractlen as usize {
            a[i] = d[i]*d[i];
        }
    }

    fn generate_reflection_coefficients(&mut self) {
        let mut a = &mut self.areas;
        let mut r = &mut self.reflections;
        for i in 1 .. self.tractlen as usize {
            if a[i] == 0.0 {
                a[i] = 0.999;
            } else {
                let den = a[i] - 1.0 + a[i];
                if den == 0.0 {
                    r[i] = 0.999;
                } else {
                    r[i] = (a[i - 1] - a[i]) / den;
                }
            }
        }
    }

    fn compute_scattering_junctions(&mut self, sig: f32) {
        let mut j_l = &mut self.junc_left;
        let mut j_r = &mut self.junc_right;

        let mut w_l = &mut self.left;
        let mut w_r = &mut self.right;
        let len = self.tractlen as usize;

        // reflection coefficients
        let glot_reflection = -0.85;
        let lip_reflection = 0.75;

        j_r[0] = w_l[0] * glot_reflection + sig;
        j_l[len - 1] = w_r[len - 1] * lip_reflection;

        // TODO: finish
    }

    fn aliasing_suppression(&mut self, sig: f32) -> f32 {
        // TODO implement
        sig
    }


    pub fn tick(&mut self, sig: f32) -> f32 {
        self.compute_areas_from_diams();
        self.generate_reflection_coefficients();
        self.compute_scattering_junctions(sig);

        let out = self.right[self.tractlen as usize - 1];

        // apply crude anti-aliasing filter (simple 1-pole)
        let out = self.aliasing_suppression(out);
        out
    }
}
