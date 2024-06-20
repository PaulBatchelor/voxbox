use std::f32::consts::PI;
use crate::Nose;
use crate::Smoother;

const SPEED_OF_SOUND: f32 = 343.0; /* m/s @ 20C */
const LIP_REFLECTION: f32 = -0.85;
const GLOTTAL_REFLECTION: f32 = 0.75;

pub struct Tract {
    // TODO: how to use dynbox instead?
    //
    // left/right delay waveguides
    pub left: Vec<f32>,
    pub right: Vec<f32>,
    pub junc_left: Vec<f32>,
    pub junc_right: Vec<f32>,
    pub areas: Vec<f32>,

    tractlen: usize,
    tractlen_max: usize,

    reflections: Vec<f32>,

    // TODO: maybe move diameters to another interface?
    // for now, it's convenient to have it here for tongue control
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
    sr: usize,
    pub tongue_smooth_amt: f32,
    tongue_x: f32,
    tongue_y: f32,
    tongue_smoother_x: Smoother,
    tongue_smoother_y: Smoother,
}

impl Tract {
    pub fn new(sr: usize, length: f32, oversample: u16) -> Self {

        let tractlen =
            (((length * 0.01) / (SPEED_OF_SOUND as f32 / (sr as f32 * oversample as f32)))).floor() + 1.0;

        let tractlen = tractlen as usize;

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
            tractlen_max: tractlen,
            sr: sr,
            tongue_smooth_amt: 0.0,
            tongue_smoother_x: Smoother::new(sr),
            tongue_smoother_y: Smoother::new(sr),
            tongue_x: 0.0,
            tongue_y: 0.0,
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

    pub fn apply_diameters(&mut self) {
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
        let glot_reflection = GLOTTAL_REFLECTION;
        let lip_reflection = LIP_REFLECTION;

        j_r[0] = w_l[0] * glot_reflection + sig;
        j_l[len - 1] = w_r[len - 1] * lip_reflection;

        let r = &self.reflections;
        for i in 1 .. self.tractlen as usize {
            let w = r[i] * (w_r[i - 1] + w_l[i]);
            j_r[i] = w_r[i - 1] - w;
            j_l[i - 1] = w_l[i] + w;
        }
    }

    fn update_waveguide(&mut self) {
        let j_l = &self.junc_left;
        let j_r = &self.junc_right;

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

        self.tongue_smoothing();
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

    fn tongue_smoothing(&mut self) {
        if self.tongue_smooth_amt > 0.0 {
            self.tongue_smoother_x.set_smooth(self.tongue_smooth_amt);
            self.tongue_smoother_y.set_smooth(self.tongue_smooth_amt);
            let tx = self.tongue_smoother_x.tick(self.tongue_x);
            let ty = self.tongue_smoother_y.tick(self.tongue_y);
            self.compute_tongue_shape(tx, ty);
        }

    }

    pub fn tick_with_nose(&mut self, nose: &mut Nose, sig: f32) -> f32 {
        let mut out = 0.0;

        self.tongue_smoothing();

        // TODO: move nose_start to somewhere else
        // 17 / 44
        let nose_start = (0.39 * self.tractlen as f32) as usize;
        // TODO: probably not in inner loop?
        //dbg!(self.tractlen);
        for _ in 0 .. self.oversample {
            self.generate_reflection_coefficients();

            if self.left[nose_start].is_nan() {
                // dbg!(nose.samppos);
                // panic!("NAN");
            }

            nose.calculate_reflections_with_tract(self, nose_start);

            // Doesn't seem to trigger a NaN
            if self.junc_left[nose_start].is_nan() {
                // This appears to be the earliest NaN at 1366
                //dbg!(nose.samppos);
                //panic!("NAN");
            }

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
        let pos = pos.clamp(0.0, 1.0);
        let diam = diam.clamp(0.0, 1.0);

        self.tongue_x = pos;
        self.tongue_y = diam;

        if self.tongue_smooth_amt > 0.0 {
            // self.tongue_smoother_x.snap_to_value(self.tongue_x);
            // self.tongue_smoother_y.snap_to_value(self.tongue_y);
            return;
        }

        self.compute_tongue_shape(pos, diam);
    }

    pub fn set_tongue_smooth(&mut self, smooth: f32) {
        if self.tongue_smooth_amt <= 0.0 {
            self.tongue_smoother_x.snap_to_value(self.tongue_x);
            self.tongue_smoother_y.snap_to_value(self.tongue_y);
        }
        self.tongue_smooth_amt = smooth;
    }

    fn compute_tongue_shape(&mut self, pos: f32, diam: f32) {
        // Adapted from original PT code, which used
        // hard coded constants relative to size 44
        let tract_scaler = self.tractlen as f32 / 44.0;
        let pos = (12.0 + 16.0*pos) * tract_scaler;
        let diam = 3.5 * diam;
        let blade_start = (10.0 * tract_scaler) as usize;
        let lip_start = (39.0 * tract_scaler) as usize;
        let tip_start = (32.0 * tract_scaler) as usize;
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

        self.apply_diameters();

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

    pub fn get_lip_reflection(&self) -> f32 {
        return LIP_REFLECTION;
    }

    pub fn set_length(&mut self, len_cm: f32) {
        let tractlen =
            (((len_cm * 0.01) /
              (SPEED_OF_SOUND as f32 /
               (self.sr as f32 * self.oversample as f32)))).floor() + 1.0;

        let mut tractlen = tractlen as usize;

        if tractlen > self.tractlen_max {
            tractlen = self.tractlen_max;
        }

        self.tractlen = tractlen;
    }
}
