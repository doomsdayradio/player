use rustfft::{num_complex::Complex, Fft, FftPlanner};
use std::sync::Arc;

pub const FFT_SIZE: usize = 2048;
pub const BANDS: usize = 48;

pub struct Spectrum {
    fft: Arc<dyn Fft<f32>>,
    buffer: Vec<Complex<f32>>,
    scratch: Vec<Complex<f32>>,
    pub levels: [f32; BANDS],
}

impl Spectrum {
    pub fn new() -> Self {
        let fft = FftPlanner::new().plan_fft_forward(FFT_SIZE);
        let scratch = vec![Complex::default(); fft.get_inplace_scratch_len()];
        Self {
            fft,
            buffer: vec![Complex::default(); FFT_SIZE],
            scratch,
            levels: [0.0; BANDS],
        }
    }

    pub fn update(&mut self, samples: &[f32], sample_rate: u32, gain: f32, elapsed: f32) {
        let mut targets = [0.0_f32; BANDS];
        if samples.len() == FFT_SIZE && sample_rate > 0 && gain > 0.0 {
            for (index, (value, sample)) in self.buffer.iter_mut().zip(samples).enumerate() {
                let window =
                    0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / FFT_SIZE as f32).cos();
                *value = Complex::new(sample * window * gain, 0.0);
            }
            self.fft
                .process_with_scratch(&mut self.buffer, &mut self.scratch);
            let upper = 16_000.0_f32.min(sample_rate as f32 / 2.0);
            for (band, target) in targets.iter_mut().enumerate() {
                let lower_hz = 40.0 * (upper / 40.0).powf(band as f32 / BANDS as f32);
                let upper_hz = 40.0 * (upper / 40.0).powf((band + 1) as f32 / BANDS as f32);
                let start = ((lower_hz * FFT_SIZE as f32 / sample_rate as f32) as usize).max(1);
                let end = ((upper_hz * FFT_SIZE as f32 / sample_rate as f32).ceil() as usize)
                    .max(start + 1)
                    .min(FFT_SIZE / 2);
                let magnitude = self.buffer[start..end]
                    .iter()
                    .map(|value| value.norm())
                    .fold(0.0_f32, f32::max)
                    * 4.0
                    / FFT_SIZE as f32;
                let decibels = 20.0 * magnitude.max(1e-6).log10();
                *target = ((decibels + 60.0) / 60.0).clamp(0.0, 1.0);
            }
        }
        for (level, target) in self.levels.iter_mut().zip(targets) {
            let speed = if target > *level { 24.0 } else { 6.0 };
            *level += (target - *level) * (1.0 - (-speed * elapsed).exp());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_has_no_spectrum() {
        let mut spectrum = Spectrum::new();
        spectrum.update(&[0.0; FFT_SIZE], 44_100, 1.0, 1.0);
        assert!(spectrum.levels.iter().all(|level| *level == 0.0));
    }

    #[test]
    fn locates_a_one_kilohertz_tone() {
        let samples: Vec<_> = (0..FFT_SIZE)
            .map(|index| (std::f32::consts::TAU * 1_000.0 * index as f32 / 44_100.0).sin() * 0.5)
            .collect();
        let mut spectrum = Spectrum::new();
        spectrum.update(&samples, 44_100, 1.0, 1.0);
        let strongest = spectrum
            .levels
            .iter()
            .enumerate()
            .max_by(|left, right| left.1.total_cmp(right.1))
            .unwrap()
            .0;
        let expected =
            ((1_000.0_f32 / 40.0).ln() / (16_000.0_f32 / 40.0).ln() * BANDS as f32) as usize;
        assert!(strongest.abs_diff(expected) <= 1);
        assert!(spectrum.levels[strongest] > 0.8);
    }

    #[test]
    fn mute_decays_to_silence() {
        let mut spectrum = Spectrum::new();
        spectrum.levels.fill(1.0);
        spectrum.update(&[1.0; FFT_SIZE], 44_100, 0.0, 2.0);
        assert!(spectrum.levels.iter().all(|level| *level < 0.001));
    }
}
