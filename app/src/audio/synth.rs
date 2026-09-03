//! Pure waveform synthesis for the game sounds — no macroquad, unit-testable.
//!
//! Each generator returns a mono 16-bit PCM WAV byte buffer that the audio layer
//! loads once at startup. The parameters (frequencies, durations, the building
//! flip chord, the ~10 Hz rotor chop) are ported from the original's Sound
//! Manager setup; see the `stuntcopter-flow-sound` notes.

/// Output sample rate. Any rate works — only the tone frequencies matter — so we
/// pick a modest one to keep the generated buffers small.
const SAMPLE_RATE: u32 = 22_050;

/// The four flip-fanfare tones, in Hz (base octave), from the original's
/// FourTone rates.
const FANFARE_TONES: [f32; 4] = [39.0, 104.0, 131.0, 156.0];
/// Fanfare segments as (tones sounding, seconds): the chord builds one note at a
/// time, then holds all four — durations from the original's `FlipTime` ticks.
const FANFARE_SEGMENTS: [(usize, f32); 4] = [(1, 0.167), (2, 0.083), (3, 0.083), (4, 0.333)];
/// Highest level the fanfare rises for; the octave doubles each level up to here.
pub const MAX_LEVEL: i32 = 5;

/// The looping copter drone: ~10 rotor chops a second over a quiet noise bed.
#[must_use]
pub fn drone() -> Vec<u8> {
    // A one-second loop holds a whole number of 10 Hz chops, so it repeats
    // seamlessly.
    let len = SAMPLE_RATE as usize;
    let chop = SAMPLE_RATE as usize / 10;
    let mut noise = Noise::new();
    let samples: Vec<f32> = (0..len)
        .map(|i| {
            // Each chop is a sharp burst of noise that decays over its period.
            let phase = (i % chop) as f32 / chop as f32;
            let amp = 0.08 + 0.5 * (-phase * 12.0).exp();
            noise.next() * amp
        })
        .collect();
    wav(&samples)
}

/// The splat: a short, low sawtooth buzz (~43.5 Hz) that decays away.
#[must_use]
pub fn splat() -> Vec<u8> {
    let len = (SAMPLE_RATE as f32 * 0.13) as usize;
    let freq = 43.5;
    let samples: Vec<f32> = (0..len)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let saw = (t * freq).fract() * 2.0 - 1.0;
            let decay = 1.0 - i as f32 / len as f32;
            saw * decay * 0.6
        })
        .collect();
    wav(&samples)
}

/// The success fanfare for `level`: the building square-wave chord, pitched up
/// one octave per level and capped at [`MAX_LEVEL`].
#[must_use]
pub fn fanfare(level: i32) -> Vec<u8> {
    let octave = 2f32.powi(level.clamp(1, MAX_LEVEL) - 1);
    let total: usize = FANFARE_SEGMENTS
        .iter()
        .map(|&(_, secs)| (SAMPLE_RATE as f32 * secs) as usize)
        .sum();

    let mut samples = Vec::with_capacity(total);
    for &(tones, secs) in &FANFARE_SEGMENTS {
        let len = (SAMPLE_RATE as f32 * secs) as usize;
        for _ in 0..len {
            // Continuous time across segments keeps each square's phase smooth
            // as new tones join the chord.
            let t = samples.len() as f32 / SAMPLE_RATE as f32;
            let chord: f32 = FANFARE_TONES[..tones]
                .iter()
                .map(|&f| square(f * octave, t))
                .sum();
            samples.push(chord / 4.0 * envelope(samples.len(), total) * 0.5);
        }
    }
    wav(&samples)
}

/// A square wave at `freq`, evaluated at time `t` seconds, in `[-1, 1]`.
fn square(freq: f32, t: f32) -> f32 {
    if (t * freq).fract() < 0.5 {
        1.0
    } else {
        -1.0
    }
}

/// A short attack/release envelope so a buffer doesn't click on start/stop.
fn envelope(i: usize, len: usize) -> f32 {
    let fade = (SAMPLE_RATE as usize / 200).max(1); // ~5 ms
    if i < fade {
        i as f32 / fade as f32
    } else if i + fade >= len {
        (len - i) as f32 / fade as f32
    } else {
        1.0
    }
}

/// Wrap mono `f32` samples (`[-1, 1]`) as a 16-bit PCM WAV byte buffer.
fn wav(samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // PCM fmt chunk size
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // byte rate
    out.extend_from_slice(&2u16.to_le_bytes()); // block align
    out.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for &s in samples {
        out.extend_from_slice(&((s.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16).to_le_bytes());
    }
    out
}

/// A tiny xorshift32 noise source, so synthesis needs no external RNG.
struct Noise(u32);

impl Noise {
    fn new() -> Self {
        Self(0x1234_5678)
    }

    /// The next noise sample in `[-1, 1)`.
    fn next(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::{drone, fanfare, splat, SAMPLE_RATE};

    /// Bytes of a well-formed mono 16-bit WAV of `frames` samples.
    fn wav_frames(bytes: &[u8]) -> usize {
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        (bytes.len() - 44) / 2
    }

    #[test]
    fn generators_produce_wav_of_the_expected_length() {
        assert_eq!(wav_frames(&drone()), SAMPLE_RATE as usize); // 1 s loop
        assert!(wav_frames(&splat()) > 0);
        // The fanfare's four segments total ~0.666 s.
        let frames = wav_frames(&fanfare(1));
        let expected = (SAMPLE_RATE as f32 * 0.666) as usize;
        assert!((frames as i32 - expected as i32).abs() < 4);
    }

    #[test]
    fn every_level_has_a_fanfare_and_it_stays_bounded() {
        for level in 1..=6 {
            let bytes = fanfare(level); // level 6 clamps to the level-5 octave
            assert!(wav_frames(&bytes) > 0);
        }
    }
}
