//! Radical (部首) collator. Phase 3.1 Stream C.
//!
//! This is a placeholder. The actual implementation will land via a
//! parallel worktree branch (`feat/collator-radical`); see CONTRIBUTING.md
//! for the recipe.
//!
//! Until then, [`RadicalCollator`] returns `None` for every character,
//! so any sort under this collator falls back to the unmapped/character
//! tiebreak path. The struct exists so [`crate::AnyCollator::Radical`]
//! can compile when the `collator-radical` feature is enabled.

use crate::collator::Collator;

/// Placeholder collator. To be filled in by Phase 3.1 Stream C.
#[derive(Debug, Clone, Copy, Default)]
pub struct RadicalCollator;

impl RadicalCollator {
    pub fn new() -> Self {
        Self
    }
}

impl Collator for RadicalCollator {
    type Data = u32;

    fn data_for(&self, _character: char) -> Option<u32> {
        None
    }
}
