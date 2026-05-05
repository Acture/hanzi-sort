//! Cantonese Jyutping (粤拼) collator. Phase 3.1 Stream A.
//!
//! This is a placeholder. The actual implementation will land via a
//! parallel worktree branch (`feat/collator-jyutping`); see CONTRIBUTING.md
//! for the recipe.
//!
//! Until then, [`JyutpingCollator`] returns `None` for every character,
//! so any sort under this collator falls back to the unmapped/character
//! tiebreak path. The struct exists so [`crate::AnyCollator::Jyutping`]
//! can compile when the `collator-jyutping` feature is enabled.

use crate::collator::Collator;

/// Placeholder collator. To be filled in by Phase 3.1 Stream A.
#[derive(Debug, Clone, Copy, Default)]
pub struct JyutpingCollator;

impl JyutpingCollator {
    pub fn new() -> Self {
        Self
    }
}

impl Collator for JyutpingCollator {
    type Data = u128;

    fn data_for(&self, _character: char) -> Option<u128> {
        None
    }
}
