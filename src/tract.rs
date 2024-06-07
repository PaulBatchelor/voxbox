use std::f32::consts::PI;

const SPEED_OF_SOUND: f32 = 343.0; /* m/s @ 20C */
const LIP_REFLECTION: f32 = -0.75;

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

pub struct Tract {
    // TODO: how to use dynbox instead?
    // left/right delay waveguides
    left: Vec<f32>,
    right: Vec<f32>,
    junc_left: Vec<f32>,
    junc_right: Vec<f32>,

    tractlen: usize,

    areas: Vec<f32>,
    reflections: Vec<f32>,

    // TODO: move diameters to another interface?
    // task id: create-diams-interface
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
    pub fn new(sr: usize, length: f32, oversample: u16) -> Self {

        let tractlen =
            (((length * 0.01) / (SPEED_OF_SOUND as f32 / (sr as f32 * oversample as f32)))).floor() + 1.0;

        let tractlen = tractlen as usize;
        println!("tract len: {}", tractlen);

        let mut tr = Tract {
            areas: vec![0.0; tractlen],
            left: vec![0.0; tractlen],
            right: vec![0.0; tractlen],
            junc_left: vec![0.0; tractlen],
            junc_right: vec![0.0; tractlen],
            diams: vec![0.0; tractlen],
            reflections: vec![0.0; tractlen],
            c1: 0.0,
            c2: 0.0,
            hp: 0.0,
            yt1: 0.0,
            oversample: oversample,
            prvhp: 0.0,
            tpidsr: 2.0 * PI / (sr as f32 * oversample as f32),
            tractlen: tractlen,
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

    // TODO: maybe take in diameters as an argument?
    #[allow(dead_code)]
    fn compute_areas_from_diams(&mut self) {
        let a = &mut self.areas;
        let d = &mut self.diams;

        for i in 0 .. self.tractlen as usize {
            a[i] = d[i]*d[i];
        }
    }

    fn generate_reflection_coefficients(&mut self) {
        let a = &mut self.areas;
        let r = &mut self.reflections;
        for i in 1 .. self.tractlen as usize {
            if a[i] == 0.0 {
                a[i] = 0.999;
            } else {
                let den = a[i - 1] + a[i];
                if den == 0.0 {
                    r[i] = 0.999;
                } else {
                    r[i] = (a[i - 1] - a[i]) / den;
                }
            }
        }
    }

    fn compute_scattering_junctions(&mut self, sig: f32) {
        let j_l = &mut self.junc_left;
        let j_r = &mut self.junc_right;

        let w_l = &mut self.left;
        let w_r = &mut self.right;
        let len = self.tractlen as usize;

        // reflection coefficients
        let glot_reflection = -0.85;
        let lip_reflection = LIP_REFLECTION;

        j_r[0] = w_l[0] * glot_reflection + sig;
        j_l[len - 1] = w_r[len - 1] * lip_reflection;

        let r = &self.reflections;
        for i in 1 .. self.tractlen as usize {
            let w = r[i] * (w_r[i - 1] + w_l[i]);
            j_r[i] = w_r[i - 1] - w;
            j_l[i - 1] = w_l[i] + w;
        }

        // TODO: nasal computation needs to go here
        // before waveguide update below

        // for i in 0 .. self.tractlen as usize {
        //     w_r[i] = j_r[i] * 0.999;
        //     w_l[i] = j_l[i] * 0.999;
        // }
    }

    fn update_waveguide(&mut self) {
        let j_l = &mut self.junc_left;
        let j_r = &mut self.junc_right;

        let w_l = &mut self.left;
        let w_r = &mut self.right;
        for i in 0 .. self.tractlen as usize {
            w_r[i] = j_r[i] * 0.999;
            w_l[i] = j_l[i] * 0.999;
        }
    }

    fn aliasing_suppression(&mut self, sig: f32) -> f32 {
        self.yt1 = self.c1*sig + self.c2*self.yt1;
        self.yt1
    }


    pub fn tick(&mut self, sig: f32) -> f32 {
        let mut out = 0.0;

        for _ in 0 .. self.oversample {
            //self.compute_areas_from_diams();
            self.generate_reflection_coefficients();
            self.compute_scattering_junctions(sig);
            self.update_waveguide();

            out = self.right[self.tractlen as usize - 1];

            // apply crude anti-aliasing filter (simple 1-pole)
            out = self.aliasing_suppression(out);
        }

        out
    }

    pub fn tick_with_nose(&mut self, nose: &mut Nose, sig: f32) -> f32 {
        let mut out = 0.0;

        // TODO: move nose_start to somewhere else
        // 17 / 44
        let nose_start = (0.38 * self.tractlen as f32) as usize;
        for _ in 0 .. self.oversample {
            self.generate_reflection_coefficients();

            // should be called after generating reflections
            nose.calculate_reflections_with_tract(self, nose_start);

            self.compute_scattering_junctions(sig);
    
            let nasal = nose.tick(self, nose_start);

            self.update_waveguide();

            out = self.right[self.tractlen as usize - 1];
            out += nasal;

            // apply crude anti-aliasing filter (simple 1-pole)
            out = self.aliasing_suppression(out);
        }

        // TODO: apply nasal component with velum control 
        out
    }

    pub fn tongue_shape(&mut self, pos: f32, diam: f32) {
        let pos = 12.0 + 16.0*pos;
        let diam = 3.5 * diam;
        let blade_start = 10;
        let lip_start = 39;
        let tip_start = 32;
        let tip_blade_delta = (tip_start - blade_start) as f32;
        let fixed_tongue_diam = 2.0 + (diam - 2.0) / 1.5;

        for i in blade_start .. lip_start {
            let t =
                1.1 * PI *
                (pos - i as f32) / tip_blade_delta;
            let mut curve = (1.5 - fixed_tongue_diam) * t.cos();

            if i == blade_start - 2 || i == lip_start - 1 {
                curve *= 0.8;
            }

            if i == blade_start || i == lip_start - 2 {
                curve *= 0.94;
            }

            self.diams[i] = 1.5 - curve;
        }

    }

    pub fn drm(&mut self, regions: &[f32]) {
        let tractlen = self.tractlen as f32;

        let l_10 = (tractlen / 10.0) as usize;
        let l_15 = (tractlen / 15.0) as usize;
        let l_5 = (tractlen / 5.0) as usize;
        let mut pos = 0;

        let areas = &mut self.areas;

        for _ in 0 .. l_10 {
            areas[pos] = regions[0];
            pos += 1;
        }

        for _ in 0 .. l_15 {
            areas[pos] = regions[1];
            pos += 1;
        }

        for _ in 0 .. 2*l_15 {
            areas[pos] = regions[2];
            pos += 1;
        }

        for _ in 0 .. l_5 {
            areas[pos] = regions[3];
            pos += 1;
        }

        for _ in 0 .. l_5 {
            areas[pos] = regions[4];
            pos += 1;
        }

        for _ in 0 .. 2*l_15 {
            areas[pos] = regions[5];
            pos += 1;
        }

        for _ in 0 .. l_15 {
            areas[pos] = regions[6];
            pos += 1;
        }

        for _ in 0 .. l_10 {
            areas[pos] = regions[7];
            pos += 1;
        }

        while pos < self.tractlen {
            areas[pos] = regions[7];
            pos += 1;
        }

    }
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

            if d > 1.9 {
                d = 1.9;
            }

            diams[i] = d;
        }
        self.calculate_reflections();

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


    pub fn calculate_reflections_with_tract(&mut self, tr: &mut Tract, nose_start: usize)
    {
        self.diams[0] = self.velum;
        self.areas[0] = self.diams[0]*self.diams[0];
        let sum =
            tr.areas[nose_start] +
            tr.areas[nose_start + 1] +
            self.areas[0];
        self.reflection_left = (2.0 * tr.areas[nose_start] - sum) / sum;
        self.reflection_right = (2.0 * tr.areas[nose_start] + sum) / sum;
        self.reflection_nose = (2.0 * self.areas[0] - sum) / sum;
    }

    pub fn tick(&mut self, tr: &mut Tract, nose_start: usize) -> f32 {
        let r = self.reflection_left;
        tr.junc_left[nose_start - 1] =
            r*tr.right[nose_start - 1] +
            (1.0 + r)*(self.left[0] + tr.left[nose_start]);

        let r = self.reflection_right;
        // TODO check this equation, it looks wrong.
        // shouldn't it match junc_left more?
        tr.junc_right[nose_start] =
            r*tr.left[nose_start] +
            (1.0 + r)*(self.left[0] + tr.right[nose_start - 1]);

        let r = self.reflection_nose;
        self.junc_right[0] =
            r*self.left[0] +
            (1.0+r)*(tr.left[nose_start]+tr.right[nose_start - 1]);

        self.junc_left[self.length - 1] =
            LIP_REFLECTION*self.right[self.length - 1];

        for i in 1 .. self.length {
            let w =
                self.reflections[i] * 
                (self.right[i - 1] + self.left[i]);
            self.junc_right[i] = self.right[i - 1] - w;
            self.junc_left[i] = self.left[i] + w;
        }

        self.right[self.length - 1] * self.velum
    }
}
