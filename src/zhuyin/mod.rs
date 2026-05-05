//! Mandarin Zhuyin (注音 / Bopomofo) collator. Phase 3.1 Stream B.
//!
//! Sorts Hanzi by the primary Mandarin Zhuyin reading generated from the
//! bundled pinyin data. The generated key uses canonical Bopomofo order, not
//! raw Unicode codepoint order.

use crate::collator::Collator;

/// Sorts Hanzi by Mandarin Zhuyin / Bopomofo reading.
#[derive(Debug, Clone, Copy, Default)]
pub struct ZhuyinCollator;

impl ZhuyinCollator {
    pub fn new() -> Self {
        Self
    }
}

impl Collator for ZhuyinCollator {
    type Data = u128;

    fn data_for(&self, ch: char) -> Option<u128> {
        crate::generated::zhuyin_map::ZHUYIN_MAP
            .get(&(ch as u32))
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collator::sort_strings_with;

    #[test]
    fn data_for_returns_known_zhuyin() {
        let collator = ZhuyinCollator::new();
        let han = collator.data_for('汉').expect("汉 should have zhuyin data");
        let zhong = collator.data_for('中').expect("中 should have zhuyin data");
        let yi = collator.data_for('一').expect("一 should have zhuyin data");

        assert!(han < zhong, "ㄏ should sort before ㄓ");
        assert!(zhong < yi, "initial ㄓ should sort before medial ㄧ");
        assert_ne!(han, yi);
    }

    #[test]
    fn sort_orders_by_zhuyin_canonical_order() {
        let collator = ZhuyinCollator::new();
        let sorted = sort_strings_with(vec!["大".into(), "八".into()], &collator);
        assert_eq!(sorted, vec!["八", "大"]);
    }
}
