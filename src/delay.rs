pub struct Delay {
    buf: Vec<f32>,
    pos: usize,
    // fdbk: f32,
}

impl Delay {
    pub fn new(sr: usize, delay_time_s: f32) -> Self {
        let bufsize = (delay_time_s * sr as f32) as usize;
        Delay {
            buf: vec![0.0; bufsize],
            pos: 0,
        }
    }

    pub fn tick(&mut self, sig: f32) -> f32 {
        self.buf[self.pos] = sig;
        self.pos += 1;
        if self.pos >= self.buf.len() {
            self.pos = 0;
        }
        self.buf[self.pos]
    }
}
