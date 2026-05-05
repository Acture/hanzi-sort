//! Shared test data for hanzi-sort benchmarks.
//!
//! The data is deterministic (no RNG) so two benchmark runs on the same
//! commit see identical inputs. Inputs are derived from a fixed list of
//! ~100 high-frequency Hanzi by taking pairs and triples; this gives
//! varied but reproducible workloads at multiple scales.

#![allow(dead_code)]

/// 100 of the most common Hanzi (taken from a frequency list of modern
/// Mandarin Chinese). Sufficient diversity for benchmarks; deliberately
/// small to keep the benchmark startup cheap.
pub const COMMON_HANZI: &[char] = &[
    '的', '一', '是', '不', '了', '在', '人', '有', '我', '他',
    '这', '个', '们', '中', '来', '上', '大', '为', '和', '国',
    '地', '到', '以', '说', '时', '要', '就', '出', '会', '可',
    '也', '你', '对', '生', '能', '而', '子', '那', '得', '于',
    '着', '下', '自', '之', '年', '过', '发', '后', '作', '里',
    '用', '道', '行', '所', '然', '家', '种', '事', '成', '方',
    '多', '经', '么', '去', '法', '学', '如', '都', '同', '现',
    '当', '没', '动', '面', '起', '看', '定', '天', '分', '还',
    '进', '好', '小', '部', '其', '些', '主', '样', '理', '心',
    '她', '本', '前', '开', '但', '因', '只', '从', '想', '实',
];

/// Generate `n` distinct 2-character Chinese strings deterministically
/// (cycling pairs from `COMMON_HANZI`). When `n > 100*100`, the sequence
/// repeats — use `triples_subset` for bigger workloads.
pub fn pairs(n: usize) -> Vec<String> {
    let len = COMMON_HANZI.len();
    (0..n)
        .map(|i| {
            let a = COMMON_HANZI[(i / len) % len];
            let b = COMMON_HANZI[i % len];
            format!("{a}{b}")
        })
        .collect()
}

/// Generate `n` distinct 3-character Chinese strings deterministically.
pub fn triples_subset(n: usize) -> Vec<String> {
    let len = COMMON_HANZI.len();
    (0..n)
        .map(|i| {
            let a = COMMON_HANZI[(i / (len * len)) % len];
            let b = COMMON_HANZI[(i / len) % len];
            let c = COMMON_HANZI[i % len];
            format!("{a}{b}{c}")
        })
        .collect()
}

/// All single-character inputs from the common-Hanzi pool. Stresses the
/// per-character lookup path more than the per-string sort logic.
pub fn singles() -> Vec<String> {
    COMMON_HANZI.iter().map(|c| c.to_string()).collect()
}
