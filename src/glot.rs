const GLOT_ENV_SIZE: usize = 512;

pub struct GlotFilt {
    freq: f32,
    lfreq: f32,
    a: [f32; 7],
    pidsr: f32,
    tpidsr: f32
} 

pub struct Glot {
    freq: f32,
    r_d: f32,
    waveform_length: f32,

    /* Pulsed Noise */
    alpha: f32,
    e_0: f32,
    epsilon: f32,
    shift: f32,
    delta: f32,
    t_e: f32,
    omega: f32,

    t: f32,
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

    asp_hpfilt: GlotFilt,
    asp_lpfilt: GlotFilt,
}

impl Glot {
    // TODO
    // new
    // freq
    // srand
    // tick
    // shape
    // aspiration
    // noise_floor
}

impl GlotFilt {
    // TODO
    // new
    // tick_hp
    // tick_lp
}
