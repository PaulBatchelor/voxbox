// BigVerb
// Waveguide FDN Reverberator
// based on: https://pbat.ch/sndkit/bigverb/

use std::f32::consts::PI;

#[derive(Clone, Copy)]
struct ParamSet {
    delay: i32,
    drift: i32,
    randfreq: i32,
    seed: i32,
}

const PARAMS: [ParamSet; 8] = [
    ParamSet {
        delay: 0x09a9,
        drift: 0x0a,
        randfreq: 0xc1c,
        seed: 0x07ae,
    },
    ParamSet {
        delay: 0x0acf,
        drift: 0x0b,
        randfreq: 0xdac,
        seed: 0x7333,
    },
    ParamSet {
        delay: 0x0c91,
        drift: 0x11,
        randfreq: 0x456,
        seed: 0x5999,
    },
    ParamSet {
        delay: 0x0de5,
        drift: 0x06,
        randfreq: 0xf85,
        seed: 0x2666,
    },
    ParamSet {
        delay: 0x0f43,
        drift: 0x0a,
        randfreq: 0x925,
        seed: 0x50a3,
    },
    ParamSet {
        delay: 0x101f,
        drift: 0x0b,
        randfreq: 0x769,
        seed: 0x5999,
    },
    ParamSet {
        delay: 0x085f,
        drift: 0x11,
        randfreq: 0x37b,
        seed: 0x7333,
    },
    ParamSet {
        delay: 0x078d,
        drift: 0x06,
        randfreq: 0xc95,
        seed: 0x3851,
    },
];

const FRACSCALE: u32 = 0x10000000;
const FRACMASK: u32 = 0xFFFFFFF;
const FRACNBITS: u32 = 28;

#[derive(Clone, Copy)]
struct BigVerbDelay {
    bufstart: usize,
    sz: usize,
    wpos: i32,
    irpos: i32,
    frpos: i32,
    rng: i32,
    inc: i32,
    counter: i32,
    maxcount: i32,
    dels: f32,
    drift: f32,
    y: f32,
}

pub struct BigVerb {
    sr: usize,
    pub size: f32,
    cutoff: f32,
    pcutoff: f32,
    filt: f32,
    delay: [BigVerbDelay; 8],
    buf: Vec<f32>,
}

fn get_delay_size(p: &ParamSet, sr: usize) -> usize {
    let sz = p.delay as f32 / 44100.0 + (p.drift as f32 * 0.0001) * 1.125;
    (16.0 + sz * sr as f32).floor() as usize
}

impl BigVerbDelay {
    pub fn new() -> Self {
        BigVerbDelay {
            bufstart: 0,
            sz: 0,
            wpos: 0,
            irpos: 0,
            frpos: 0,
            rng: 0,
            inc: 0,
            counter: 0,
            maxcount: 0,
            dels: 0.0,
            drift: 0.0,
            y: 0.0,
        }
    }

    pub fn init(&mut self, bufstart: usize, sz: usize, p: &ParamSet, sr: usize) {
        let readpos = p.delay as f32 / 44100.0;
        let readpos = readpos + p.seed as f32 * (p.drift as f32 * 0.0001) / 32768.0;
        let readpos = (sz as f32 - (readpos * sr as f32)).floor();

        self.bufstart = bufstart;
        self.sz = sz;
        self.wpos = 0;
        self.irpos = readpos.floor() as i32;
        self.frpos = ((readpos - self.irpos as f32) * FRACSCALE as f32).floor() as i32;
        self.rng = p.seed;
        self.inc = 0;
        self.counter = 0;
        self.dels = p.delay as f32 / 44100.0;
        self.drift = p.drift as f32;
        self.maxcount = (sr as f32 / (p.randfreq as f32 * 0.001)).floor() as i32;
        self.generate_next_line(sr);
        self.y = 0.0;
    }

    pub fn generate_next_line(&mut self, sr: usize) {
        if self.rng < 0 {
            self.rng += 0x10000;
        }

        self.rng = 1 + self.rng * 0x3d09;
        self.rng &= 0xFFFF;

        if self.rng >= 0x8000 {
            self.rng -= 0x10000;
        }

        self.counter = self.maxcount;

        let mut curdel =
            self.wpos as f32 - (self.irpos as f32 + (self.frpos as f32 / FRACSCALE as f32));

        while curdel < 0.0 {
            curdel += self.sz as f32;
        }

        curdel /= sr as f32;

        let nxtdel = (self.rng as f32 * (self.drift * 0.0001) / 32768.0) + self.dels;

        let inc = ((curdel - nxtdel) / self.counter as f32) * sr as f32;
        let inc = inc + 1.0;
        self.inc = (inc * FRACSCALE as f32).floor() as i32;
    }

    pub fn tick(&mut self, buf: &mut [f32], insig: f32, fdbk: f32, filt: f32, sr: usize) -> f32 {
        let bufpos = self.bufstart as i32 + self.wpos;
        buf[bufpos as usize] = insig - self.y;
        self.wpos += 1;
        if self.wpos >= self.sz as i32 {
            self.wpos -= self.sz as i32;
        }
        if self.frpos >= FRACSCALE as i32 {
            self.irpos += self.frpos >> FRACNBITS;
            self.frpos &= FRACMASK as i32;
        }
        if self.irpos >= self.sz as i32 {
            self.irpos -= self.sz as i32;
        }
        let frac_norm = self.frpos as f32 / FRACSCALE as f32;

        let d = ((frac_norm * frac_norm) - 1.0) / 6.0;
        let t1 = (frac_norm + 1.0) * 0.5;
        let t2 = 3.0 * d;
        let a = t1 - 1.0 - d;
        let c = t1 - t2;
        let b = t2 - frac_norm;

        let mut n = self.irpos;
        let mut s: [f32; 4] = [0.0; 4];
        let bufoff = self.bufstart + n as usize;
        if n > 0 && n < (self.sz - 2).try_into().unwrap() {
            s[0] = buf[bufoff - 1];
            s[1] = buf[bufoff];
            s[2] = buf[bufoff + 1];
            s[3] = buf[bufoff + 2];
        } else {
            n -= 1;
            if n < 0 {
                n += self.sz as i32;
            }
            s[0] = buf[self.bufstart + n as usize];
            for k in 0..3 {
                n += 1;
                if n >= self.sz as i32 {
                    n -= self.sz as i32;
                }
                s[k + 1] = buf[self.bufstart + n as usize];
            }
        }

        let out = (a * s[0] + b * s[1] + c * s[2] + d * s[3]) * frac_norm + s[1];
        self.frpos += self.inc;
        let out = out * fdbk;
        let out = out + (self.y - out) * filt;
        self.y = out;
        self.counter -= 1;

        if self.counter <= 0 {
            self.generate_next_line(sr)
        }

        out
    }
}

impl BigVerb {
    pub fn new(sr: usize) -> Self {
        let mut bufsize = 0;

        for p in PARAMS {
            bufsize += get_delay_size(&p, sr);
        }

        BigVerb {
            sr,
            size: 0.93,
            cutoff: 10000.0,
            pcutoff: -1.0,
            filt: 1.0,
            delay: [BigVerbDelay::new(); 8],
            buf: vec![0.0; bufsize],
        }
    }

    pub fn init(&mut self) {
        let mut bufpos = 0;
        for (i, param) in PARAMS.iter().enumerate() {
            let bufsz = get_delay_size(param, self.sr);
            self.delay[i].init(bufpos, bufsz, param, self.sr);
            bufpos += bufsz;
        }
    }

    pub fn tick(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        if self.pcutoff != self.cutoff {
            self.pcutoff = self.cutoff;
            //bv->filt = 2.0 - cos(bv->pcutoff * 2 * M_PI / bv->sr);
            self.filt = 2.0 - (self.cutoff * 2.0 * PI / self.sr as f32).cos();
            //bv->filt = bv->filt - sqrt(bv->filt * bv->filt - 1.0);
            self.filt = self.filt - (self.filt * self.filt - 1.0).sqrt();
        }

        let mut jp = 0.0;

        for i in 0..8 {
            jp += self.delay[i].y;
        }

        jp *= 0.25;

        let in_l = jp + in_l;
        let in_r = jp + in_r;

        let mut lsum = 0.0;
        let mut rsum = 0.0;

        for i in 0..8 {
            if (i & 1) > 0 {
                rsum += self.delay[i].tick(&mut self.buf, in_r, self.size, self.filt, self.sr);
            } else {
                lsum += self.delay[i].tick(&mut self.buf, in_l, self.size, self.filt, self.sr);
            }
        }
        rsum *= 0.35;
        lsum *= 0.35;
        (lsum, rsum)
    }
}
