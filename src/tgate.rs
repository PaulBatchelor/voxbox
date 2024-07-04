pub struct TriggerGate {
    onedsr: f32,
    pub duration: f32,
    counter: f32,
}

impl TriggerGate {
    pub fn new(sr: usize) -> Self {
        TriggerGate {
            onedsr: 1.0 / sr as f32,
            duration: 0.0,
            counter: 0.0
        }
    }

    pub fn tick(&mut self, trig: f32) -> f32 {

        if trig > 0. {
            //self.counter = self.dur;
            // Counter normalized to 0-1, allows for
            // dynamically changing duration
            self.counter = 1.0;
        }

        let out = if self.counter > 0.0 {
            // Building up my intuition:
            // When duration is 1s, that's 44100 (sr) samples,
            // or 1/44100 (1/sr). When you go 2 samples
            // at a time, it's twice as fast, or 0.5s. The
            // relationship is inversely proportional,
            // so divide (instead of multiply) by duration.
            if self.duration > 0.0 {
                self.counter -= self.onedsr / self.duration;
            }
            1.0
        } else  {
            0.0
        };

        out
    }
}
