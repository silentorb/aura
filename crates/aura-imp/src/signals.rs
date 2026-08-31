//! Shared signal type identifiers for Aura Imp libraries.

use imp_core_types::SignalType;

/// Seconds from render start.
pub const TIME: &str = "time";

/// Scalar control value.
pub const CONTROL: &str = "control";

/// Mono audio sample amplitude.
pub const AUDIO_MONO: &str = "audio_mono";

/// Musical score (translate-time only).
pub const SCORE: &str = "score";

/// Chord progression (translate-time data signal).
pub const CHORD_PROGRESSION: &str = "chord_progression";

pub fn signal(id: &str) -> SignalType {
    SignalType { id: id.into() }
}
