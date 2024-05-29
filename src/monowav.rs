use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::SeekFrom;

#[allow(dead_code)]

// #[derive(Clone, Copy)]
pub struct MonoWav {
    wav: BufWriter<File>,
    nbytes: u32,
}

impl MonoWav {
    pub fn new(wavfilename: &str) -> Self {
        let wavfile = File::create(wavfilename).unwrap();
        let mut wav = BufWriter::new(wavfile);

        Self::write_master_chunk(&mut wav);
        Self::write_fmt_chunk(&mut wav);
        Self::write_data_chunk(&mut wav);

        MonoWav {
            wav: wav,
            nbytes: 0,
        }
    }

    fn write_master_chunk(wav: &mut BufWriter<File>) {
        wav.write(b"RIFF").unwrap();
        wav.write(&0u32.to_le_bytes()).unwrap();
        wav.write(b"WAVE").unwrap();
    }

    fn write_fmt_chunk(wav: &mut BufWriter<File>) {
        wav.write(b"fmt ").unwrap();

        /* chunk size 16 (0x10) */
        wav.write(&0x10u32.to_le_bytes()).unwrap();

        /* format code: WAVE_FORMAT_PCM (0x0001) */
        wav.write(&0x0001u16.to_le_bytes()).unwrap();

        /* nchannels: 1 (always mono) */
        wav.write(&0x0001u16.to_le_bytes()).unwrap();

        /* sample rate 44.1kHz */
        wav.write(&44100u32.to_le_bytes()).unwrap();

        /* bytes per second */
        wav.write(&(44100u32 * 2).to_le_bytes()).unwrap();

        /* block alignment */
        wav.write(&2u16.to_le_bytes()).unwrap();

        /* block alignment */
        wav.write(&16u16.to_le_bytes()).unwrap();
    }

    fn write_data_chunk(wav: &mut BufWriter<File>) {
        wav.write(b"data").unwrap();
        wav.write(&0u32.to_le_bytes()).unwrap();
    }

    pub fn tick(&mut self, sig: f32) {
        let isig = (sig * 32767.0) as i16;
        self.wav.write(&isig.to_le_bytes()).unwrap();
        self.nbytes += 2;
    }
}

impl Drop for MonoWav {
    fn drop(&mut self) {
        self.wav.seek(SeekFrom::Start(0x04)).unwrap();
        self.wav.write(&(self.nbytes + 16 + 16 + 4).to_le_bytes()).unwrap();

        self.wav.seek(SeekFrom::Start(0x28)).unwrap();
        self.wav.write(&self.nbytes.to_le_bytes()).unwrap();
        self.wav.flush().unwrap();
    }
}
