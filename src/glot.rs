const GLOT_ENV_SIZE: usize = 512;
const LCG_MAX: u32 = 2147483648;
use std::f32::consts::PI;
use crate::butterworth::{ButterworthLowPass, ButterworthHighPass};

// midi-to-frequency converter
fn mtof(nn: f32) -> f32 {
    let freq = (2.0_f32).powf((nn - 69.0) / 12.0) * 440.0;
    freq
}

pub struct Glot {
    freq: f32,
    r_d: f32,
    waveform_length: f32,
    time_in_waveform: f32,

    /* Pulsed Noise */
    alpha: f32,
    e_0: f32,
    epsilon: f32,
    shift: f32,
    delta: f32,
    t_e: f32,
    omega: f32,

    onedsr: f32,
    rng: u32,

    /* pulsed noise */
    hanning: [f32; GLOT_ENV_SIZE],

    // Lu suggests that scale can be fixed between
    // 40-80% of glottal wave (pg 93)
    env_size: f32, // A_n

    // lag is recommend to be between 0 and 15% of period
    lag: f32, // L
    t_env_start: f32,
    env_pos: f32,
    env_delta: f32,
    noise_floor: f32, // B_n
    aspiration: f32,

    asp_hpfilt: ButterworthHighPass,
    asp_lpfilt: ButterworthLowPass,
}

impl Glot {
    pub fn new(sr: usize) -> Self {
        let mut glt = Glot {
            freq: 140.0,
            onedsr: 1.0 / (sr as f32),
            time_in_waveform: 0.0,
            lag: 0.07, // 7% of period (max 15)
            noise_floor: 0.003,
            env_size: 0.6, // 40-80 percent
            aspiration: 0.3,
            alpha: 0.0,
            e_0: 0.0,
            epsilon: 0.0,
            shift: 0.0,
            delta: 0.0,
            env_delta: 0.0,
            env_pos: 0.0,
            omega: 0.0,
            r_d: 0.0,
            t_e: 0.0,
            rng: 0,
            t_env_start: 0.0,
            waveform_length: 0.0,
            hanning: [0.0; GLOT_ENV_SIZE],
            asp_lpfilt: ButterworthLowPass::new(sr),
            asp_hpfilt: ButterworthHighPass::new(sr),
        };

        glt.set_aspiration(0.5);
        glt.set_shape(0.5);
        glt.srand(0);
        glt.setup_waveform();
        glt.setup_hanning_table();
        glt.asp_lpfilt.set_freq(6000.0);
        glt.asp_hpfilt.set_freq(4500.0);
        glt
    }

    fn setup_waveform(&mut self) {
        let mut r_d = self.r_d;

        if r_d < 0.5 {
            r_d = 0.5;
        }

        if r_d > 2.7 {
            r_d = 2.7;
        }

        self.waveform_length = 1.0 / self.freq;

        let r_a = -0.01 + 0.048*r_d;
        let r_k = 0.224 + 0.118*r_d;
        let r_g =
            (r_k/4.0)*(0.5 + 1.2*r_k) /
            (0.11*r_d - r_a*(0.5+1.2*r_k));

        let t_a = r_a;
        let t_p = 1.0 / (2.0 * r_g);
        let t_e = t_p + t_p*r_k;

        let epsilon = 1.0 / t_a;
        let shift = (-epsilon * (1.0 - t_e)).exp();
        let delta = 1.0 - shift;

        let rhs_integral = (1.0/epsilon) * (shift - 1.0) + (1.0 - t_e)*shift;
        let rhs_integral = rhs_integral / delta;

        let lower_integral = -(t_e - t_p) / 2.0 + rhs_integral;
        let upper_integral = -lower_integral;

        let omega = PI / t_p;
        let s = (omega * t_e).sin();

        let y = -PI * s * upper_integral / (t_p * 2.0);
        let z = y.ln();
        let alpha = z / (t_p/2.0 - t_e);
        let e_0 = -1.0 / (s * (alpha*t_e).exp());

        self.alpha = alpha;
        self.e_0 = e_0;
        self.epsilon = epsilon;
        self.shift = shift;
        self.delta = delta;
        self.t_e = t_e;
        self.omega = omega;

        // calculate envelope start from lag
        // and glottal closure (Te) (note that Te is normalized)
        // make sure to factor in delay to make it centered


        self.t_env_start =
            (self.t_e + self.lag) - 0.5*self.env_size;

        // reset envelope position
        self.env_pos = 0.0;

        // how much to increment the envelope by
        // 1/sz is a ramp sz samples long, scaled by env_size
        self.env_delta = 1.0 / (GLOT_ENV_SIZE as f32 * self.env_size);
    }

    fn setup_hanning_table(&mut self) {
        let om = 2.0 * PI / (GLOT_ENV_SIZE as f32);

        for n in 0..GLOT_ENV_SIZE {
            let out = (om*0.5*(n as f32)).sin();
            let out = out * out;
            self.hanning[n] = out;
        }
    }

    pub fn set_freq(&mut self, freq: f32) {
        self.freq = freq;
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.set_freq(mtof(pitch));
    }

    pub fn set_shape(&mut self, shape: f32) {
        self.r_d = 3.0 * (1.0 - shape);
    }

    pub fn set_aspiration(&mut self, aspiration: f32) {
        self.aspiration = aspiration;
    }

    pub fn set_noise_floor(&mut self, noise_floor: f32) {
        self.noise_floor = noise_floor;
    }

    pub fn srand(&mut self, seed: u32) {
        self.rng = seed;
    }

    fn rand(&mut self) -> u32 {
        self.rng = self.rng.wrapping_mul(1103515245);
        self.rng = self.rng.wrapping_add(12345) % LCG_MAX;
        return self.rng;
    }

    pub fn tick(&mut self) -> f32 {
        let mut out;

        self.time_in_waveform += self.onedsr;

        if self.time_in_waveform > self.waveform_length {
            self.time_in_waveform -= self.waveform_length;
            self.setup_waveform();
        }

        let t = self.time_in_waveform / self.waveform_length;

        if t > self.t_e {
            out =
                (-(-self.epsilon * (t - self.t_e)).exp() + self.shift) /
                self.delta;
        } else {
            out = self.e_0 * (self.alpha * t).exp() * (self.omega * t).sin();
        }

        // gaussian noise (more or less)
        let noise = self.rand() as f32 / LCG_MAX as f32;

        // shave off some high end
        let noise = self.asp_lpfilt.tick(noise);

        // noise filtering: lu says 4kHz highpass cutoff
        let noise = self.asp_hpfilt.tick(noise);


        // amplitude modulation
        // This is a scaled pitch-synchronous Hanning window,
        // centered on the glottal closure instants and desired lag.
        //
        // Per Lu's thesis, only one pulse per period is considered
        // as a good first approximation. The timing position
        // for the glottal closure instance is Te.
        //
        // Lag is specified as percentage relative to glottal
        // period length.
        //
        // The envelope "sits on top of the noise floor". That
        // is to say, it doesn't close all the way, letting
        // some noise out at the lower level. This is also
        // a parameter.


        let mut env = 0.0;

        // check and see if it is time to use the envelope

        if t > self.t_env_start && self.env_pos <= 1.0 {
            let fpos = self.env_pos * (GLOT_ENV_SIZE as f32 - 2.0);
            let ipos = fpos as usize;
            let fpos = fpos - ipos as f32;
            env =
                (1.0 - fpos) * self.hanning[ipos] +
                fpos * self.hanning[ipos + 1];
            self.env_pos += self.env_delta;
        }

        // noise floor / pulsed noise, this is just crossfading

        let nf = self.noise_floor;

        env = (nf + (1.0 - nf)*env) * noise;

        // attenutate by aspiration level

        env *= self.aspiration;
        out += env;
        out
    }

}
