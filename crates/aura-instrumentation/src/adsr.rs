//! Linear ADSR envelope segments.

/// Piecewise-linear ADSR envelope in seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearAdsr {
    pub attack: f64,
    pub decay: f64,
    pub sustain: f32,
    pub release: f64,
}

impl Default for LinearAdsr {
    fn default() -> Self {
        Self {
            attack: 0.005,
            decay: 0.05,
            sustain: 0.8,
            release: 0.05,
        }
    }
}

impl LinearAdsr {
    /// Envelope amplitude at `elapsed` seconds from note onset over `note_duration` seconds.
    pub fn value_at(&self, elapsed: f64, note_duration: f64) -> f32 {
        if elapsed >= note_duration || note_duration <= 0.0 {
            return 0.0;
        }

        let release_start = (note_duration - self.release).max(0.0);

        if elapsed < self.attack {
            if self.attack <= 0.0 {
                return 1.0;
            }
            return (elapsed / self.attack) as f32;
        }

        if elapsed < self.attack + self.decay {
            if self.decay <= 0.0 {
                return self.sustain;
            }
            let progress = ((elapsed - self.attack) / self.decay) as f32;
            return 1.0 + progress * (self.sustain - 1.0);
        }

        if elapsed >= release_start {
            if self.release <= 0.0 {
                return 0.0;
            }
            let progress = ((elapsed - release_start) / self.release) as f32;
            let progress = progress.min(1.0);
            return self.sustain * (1.0 - progress);
        }

        self.sustain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_starts_at_zero_and_rises() {
        let adsr = LinearAdsr::default();
        assert_eq!(adsr.value_at(0.0, 1.0), 0.0);
        assert!(adsr.value_at(0.002, 1.0) > 0.0);
    }

    #[test]
    fn envelope_reaches_zero_at_note_end() {
        let adsr = LinearAdsr::default();
        assert_eq!(adsr.value_at(1.0, 1.0), 0.0);
    }
}
