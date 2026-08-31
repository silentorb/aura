//! Aura type constraint libraries for Imp generics.

use crate::signals::{CHORD_PROGRESSION, SCORE};
use imp_core_types::{concrete_type, TypeConstraint, TypeConstraintLibrary};
use std::collections::BTreeMap;

/// Type constraint id for types that support unbounded loop (modulus over cycle length).
pub const LOOPABLE: &str = "Loopable";

/// Returns the Aura type constraint library (`Loopable` = score | chord_progression).
pub fn aura_type_constraint_library() -> TypeConstraintLibrary {
    TypeConstraintLibrary {
        id: "aura.constraints".into(),
        constraints: BTreeMap::from([(
            LOOPABLE.into(),
            TypeConstraint {
                id: LOOPABLE.into(),
                members: vec![
                    concrete_type(SCORE, vec![]),
                    concrete_type(CHORD_PROGRESSION, vec![]),
                ],
            },
        )]),
    }
}
