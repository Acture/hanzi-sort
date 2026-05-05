//! Mandarin Zhuyin (注音 / Bopomofo) collator. Phase 3.1 Stream B.
//!
//! This is a placeholder. The actual implementation will land via a
//! parallel worktree branch (`feat/collator-zhuyin`); see CONTRIBUTING.md
//! for the recipe.
//!
//! Until then, [`ZhuyinCollator`] returns `None` for every character,
//! so any sort under this collator falls back to the unmapped/character
//! tiebreak path. The struct exists so [`crate::AnyCollator::Zhuyin`]
//! can compile when the `collator-zhuyin` feature is enabled.

use crate::collator::Collator;

/// Placeholder collator. To be filled in by Phase 3.1 Stream B.
#[derive(Debug, Clone, Copy, Default)]
pub struct ZhuyinCollator;

impl ZhuyinCollator {
    pub fn new() -> Self {
        Self
    }
}

impl Collator for ZhuyinCollator {
    type Data = u128;

    fn data_for(&self, _character: char) -> Option<u128> {
        None
    }
}
