use phf_codegen::Map;
use std::collections::BTreeSet;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

fn main() {
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => dir,
        Err(_) => fail("CARGO_MANIFEST_DIR is not set"),
    };
    let project_root = Path::new(&manifest_dir);
    let data_dir = project_root.join("data");
    let data_csv = data_dir.join("pinyin.csv");
    let stroke_csv = data_dir.join("strokes.csv");
    let radical_csv = data_dir.join("radical.csv");

    // Re-run codegen when the inputs change. `build.rs` itself is included so
    // edits to the generation logic also trigger a rebuild.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", data_csv.display());
    println!("cargo:rerun-if-changed={}", stroke_csv.display());
    if std::env::var("CARGO_FEATURE_COLLATOR_RADICAL").is_ok() {
        println!("cargo:rerun-if-changed={}", radical_csv.display());
    }

    let src_dir = project_root.join("src");
    let generated_dir = src_dir.join("generated");
    if let Err(error) = std::fs::create_dir_all(&generated_dir) {
        fail(&format!(
            "failed to create generated dir {}: {error}",
            generated_dir.display()
        ));
    }

    generate_pinyin_map(&data_csv, &generated_dir.join("pinyin_map.rs"));
    generate_stroke_map(&stroke_csv, &generated_dir.join("stroke_map.rs"));

    if std::env::var("CARGO_FEATURE_COLLATOR_RADICAL").is_ok() {
        #[cfg(feature = "collator-radical")]
        generate_radical_map(&radical_csv, &generated_dir.join("radical_map.rs"));
    }

    #[cfg(feature = "collator-zhuyin")]
    {
        println!("cargo:rerun-if-env-changed=CARGO_FEATURE_COLLATOR_ZHUYIN");
        if std::env::var("CARGO_FEATURE_COLLATOR_ZHUYIN").is_ok() {
            let zhuyin_csv = data_dir.join("zhuyin.csv");
            println!("cargo:rerun-if-changed={}", zhuyin_csv.display());
            generate_zhuyin_map(&zhuyin_csv, &generated_dir.join("zhuyin_map.rs"));
        }
    }
}

fn fail(message: &str) -> ! {
    println!("cargo:warning={message}");
    eprintln!("build.rs: {message}");
    std::process::exit(1);
}

fn open_csv(path: &Path) -> csv::Reader<BufReader<File>> {
    let file = File::open(path).unwrap_or_else(|error| {
        fail(&format!("failed to open {}: {error}", path.display()));
    });
    csv::Reader::from_reader(BufReader::new(file))
}

fn create_output(path: &Path) -> BufWriter<File> {
    let file = File::create(path).unwrap_or_else(|error| {
        fail(&format!(
            "failed to create generated file {}: {error}",
            path.display()
        ));
    });
    BufWriter::new(file)
}

fn parse_codepoint(raw: &str, source: &Path, line: usize) -> u32 {
    let trimmed = raw.trim_start_matches("U+");
    u32::from_str_radix(trimmed, 16).unwrap_or_else(|error| {
        fail(&format!(
            "{}:{line}: invalid codepoint '{raw}': {error}",
            source.display()
        ));
    })
}

#[cfg(feature = "collator-zhuyin")]
const MAX_ZHUYIN_UNITS: usize = 8;

#[cfg(feature = "collator-zhuyin")]
const BOPOMOFO_RANKS: &[(char, u8)] = &[
    ('ㄅ', 1),
    ('ㄆ', 2),
    ('ㄇ', 3),
    ('ㄈ', 4),
    ('ㄉ', 5),
    ('ㄊ', 6),
    ('ㄋ', 7),
    ('ㄌ', 8),
    ('ㄍ', 9),
    ('ㄎ', 10),
    ('ㄏ', 11),
    ('ㄐ', 12),
    ('ㄑ', 13),
    ('ㄒ', 14),
    ('ㄓ', 15),
    ('ㄔ', 16),
    ('ㄕ', 17),
    ('ㄖ', 18),
    ('ㄗ', 19),
    ('ㄘ', 20),
    ('ㄙ', 21),
    ('ㄚ', 22),
    ('ㄛ', 23),
    ('ㄜ', 24),
    ('ㄝ', 25),
    ('ㄞ', 26),
    ('ㄟ', 27),
    ('ㄠ', 28),
    ('ㄡ', 29),
    ('ㄢ', 30),
    ('ㄣ', 31),
    ('ㄤ', 32),
    ('ㄥ', 33),
    ('ㄦ', 34),
    ('ㄧ', 35),
    ('ㄨ', 36),
    ('ㄩ', 37),
    ('ˊ', 38),
    ('ˇ', 39),
    ('ˋ', 40),
    ('˙', 41),
];

#[cfg(feature = "collator-zhuyin")]
fn rank_bopomofo(ch: char) -> Option<u8> {
    BOPOMOFO_RANKS
        .iter()
        .find_map(|(candidate, rank)| (*candidate == ch).then_some(*rank))
}

#[cfg(feature = "collator-zhuyin")]
fn encode_zhuyin_key(zhuyin: &str, source: &Path, line: usize, code_str: &str) -> u128 {
    let len = zhuyin.chars().count();
    if len == 0 {
        fail(&format!(
            "{}:{line}: empty zhuyin entry for {code_str}",
            source.display()
        ));
    }
    if len > MAX_ZHUYIN_UNITS {
        fail(&format!(
            "{}:{line}: zhuyin {zhuyin:?} for {code_str} exceeds {MAX_ZHUYIN_UNITS} symbols",
            source.display()
        ));
    }

    let mut encoded = 0_u128;
    for ch in zhuyin.chars() {
        let rank = rank_bopomofo(ch).unwrap_or_else(|| {
            fail(&format!(
                "{}:{line}: zhuyin {zhuyin:?} for {code_str} contains unsupported symbol {ch:?}",
                source.display()
            ));
        });
        encoded = (encoded << 16) | u128::from(rank);
    }
    encoded << ((MAX_ZHUYIN_UNITS - len) * 16)
}

fn generate_pinyin_map(data_csv: &Path, out_path: &Path) {
    let mut writer = create_output(out_path);
    let mut rdr = open_csv(data_csv);

    let mut map = Map::new();
    let mut seen_codepoints = BTreeSet::new();
    let source = PathBuf::from(data_csv);

    writeln!(
        &mut writer,
        "// This file is generated by build.rs. Do not edit manually.\n"
    )
    .unwrap_or_else(|error| fail(&format!("failed to write {}: {error}", out_path.display())));

    for (index, result) in rdr.records().enumerate() {
        let line = index + 2; // header + 1-based
        let record = result.unwrap_or_else(|error| {
            fail(&format!(
                "{}:{line}: failed to read CSV row: {error}",
                source.display()
            ));
        });
        if record.len() != 3 {
            fail(&format!(
                "{}:{line}: expected exactly 3 columns (codepoint, pinyin, char), got {}",
                source.display(),
                record.len()
            ));
        }

        let code_str = &record[0];
        let pinyins_field = &record[1];
        let char_field = &record[2];

        let code_u32 = parse_codepoint(code_str, &source, line);

        let mut chars_iter = char_field.chars();
        let character = chars_iter.next().unwrap_or_else(|| {
            fail(&format!(
                "{}:{line}: empty character column for {code_str}",
                source.display()
            ))
        });
        if chars_iter.next().is_some() {
            fail(&format!(
                "{}:{line}: character column must contain exactly one Unicode scalar (got {char_field:?})",
                source.display()
            ));
        }
        if character as u32 != code_u32 {
            fail(&format!(
                "{}:{line}: character {character:?} does not match codepoint {code_str}",
                source.display()
            ));
        }

        let pinyin_vec: Vec<&str> = pinyins_field.split('|').collect();
        if pinyin_vec.iter().any(|s| s.is_empty()) {
            fail(&format!(
                "{}:{line}: empty pinyin entry for {code_str} ({pinyins_field:?})",
                source.display()
            ));
        }

        // The primary pinyin (first reading) is what powers the fast u128
        // sort-key encoding via `encode_primary_pinyin_unchecked`. Enforce its
        // invariants at build time so the unchecked path is safe by
        // construction. Alternate readings can be non-ASCII (e.g. `ê1`) since
        // they are only exposed via `pinyin_of`, never encoded.
        let primary = pinyin_vec[0];
        if !primary.is_ascii() {
            fail(&format!(
                "{}:{line}: primary pinyin {primary:?} for {code_str} must be ASCII; \
                 normalize ü to v in the data pipeline",
                source.display()
            ));
        }
        if primary.len() > 16 {
            fail(&format!(
                "{}:{line}: primary pinyin {primary:?} for {code_str} exceeds 16 bytes",
                source.display()
            ));
        }
        match primary.as_bytes().last() {
            Some(b'1'..=b'5') => {}
            _ => fail(&format!(
                "{}:{line}: primary pinyin {primary:?} for {code_str} must end with a tone \
                 digit 1-5; the convert_pinyin_to_csv.py pipeline appends 5 for neutral/light \
                 tone — regenerate data/pinyin.csv",
                source.display()
            )),
        }

        seen_codepoints.insert(code_u32);
        let value_str = format!(
            "('{}', &[{}])",
            character,
            pinyin_vec
                .iter()
                .map(|s| format!("\"{s}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
        map.entry(code_u32, value_str.as_str());
    }

    writeln!(
        &mut writer,
        "pub static PINYIN_MAP: ::phf::Map<u32, (char, &'static [&'static str])> = {};\n",
        map.build()
    )
    .unwrap_or_else(|error| fail(&format!("failed to write {}: {error}", out_path.display())));

    for required in [0x3007_u32, 0x6C49, 0x91CD] {
        if !seen_codepoints.contains(&required) {
            fail(&format!(
                "required codepoint U+{required:04X} is missing from generated pinyin data; \
                 regenerate {} from the upstream pinyin dataset",
                source.display()
            ));
        }
    }
}

fn generate_stroke_map(data_csv: &Path, out_path: &Path) {
    let mut writer = create_output(out_path);
    let mut rdr = open_csv(data_csv);

    let mut map = Map::new();
    let mut seen_codepoints = BTreeSet::new();
    let source = PathBuf::from(data_csv);

    writeln!(
        &mut writer,
        "// This file is generated by build.rs. Do not edit manually.\n"
    )
    .unwrap_or_else(|error| fail(&format!("failed to write {}: {error}", out_path.display())));

    for (index, result) in rdr.records().enumerate() {
        let line = index + 2;
        let record = result.unwrap_or_else(|error| {
            fail(&format!(
                "{}:{line}: failed to read CSV row: {error}",
                source.display()
            ));
        });
        // The strokes CSV currently has 3 columns (codepoint, strokes, char)
        // but we only need the first two. Allow either 2 or 3 to keep the
        // pipeline forgiving while still rejecting wildly malformed rows.
        if record.len() < 2 || record.len() > 3 {
            fail(&format!(
                "{}:{line}: expected 2 or 3 columns (codepoint, strokes[, char]), got {}",
                source.display(),
                record.len()
            ));
        }

        let code_str = &record[0];
        let stroke_str = &record[1];
        let code_u32 = parse_codepoint(code_str, &source, line);
        let stroke_count: u16 = stroke_str.parse().unwrap_or_else(|error| {
            fail(&format!(
                "{}:{line}: invalid stroke count '{stroke_str}' for {code_str}: {error}",
                source.display()
            ));
        });
        if stroke_count == 0 {
            fail(&format!(
                "{}:{line}: stroke count must be greater than 0 for {code_str}",
                source.display()
            ));
        }
        seen_codepoints.insert(code_u32);
        map.entry(code_u32, &stroke_count.to_string());
    }

    writeln!(
        &mut writer,
        "pub static STROKE_MAP: ::phf::Map<u32, u16> = {};\n",
        map.build()
    )
    .unwrap_or_else(|error| fail(&format!("failed to write {}: {error}", out_path.display())));

    for required in [0x4E00_u32, 0x6C49, 0x91CD] {
        if !seen_codepoints.contains(&required) {
            fail(&format!(
                "required codepoint U+{required:04X} is missing from generated stroke data; \
                 regenerate {} from the upstream Unihan dataset",
                source.display()
            ));
        }
    }
}

#[cfg(feature = "collator-radical")]
fn generate_radical_map(data_csv: &Path, out_path: &Path) {
    let mut writer = create_output(out_path);
    let mut rdr = open_csv(data_csv);

    let mut map = Map::new();
    let mut seen_entries = BTreeSet::new();
    let source = PathBuf::from(data_csv);

    writeln!(
        &mut writer,
        "// This file is generated by build.rs. Do not edit manually.\n"
    )
    .unwrap_or_else(|error| fail(&format!("failed to write {}: {error}", out_path.display())));

    for (index, result) in rdr.records().enumerate() {
        let line = index + 2;
        let record = result.unwrap_or_else(|error| {
            fail(&format!(
                "{}:{line}: failed to read CSV row: {error}",
                source.display()
            ));
        });
        if record.len() != 4 {
            fail(&format!(
                "{}:{line}: expected exactly 4 columns (codepoint, radical, residual, char), got {}",
                source.display(),
                record.len()
            ));
        }

        let code_str = &record[0];
        let radical_str = &record[1];
        let residual_str = &record[2];
        let char_field = &record[3];
        let code_u32 = parse_codepoint(code_str, &source, line);

        let mut chars_iter = char_field.chars();
        let character = chars_iter.next().unwrap_or_else(|| {
            fail(&format!(
                "{}:{line}: empty character column for {code_str}",
                source.display()
            ))
        });
        if chars_iter.next().is_some() {
            fail(&format!(
                "{}:{line}: character column must contain exactly one Unicode scalar (got {char_field:?})",
                source.display()
            ));
        }
        if character as u32 != code_u32 {
            fail(&format!(
                "{}:{line}: character {character:?} does not match codepoint {code_str}",
                source.display()
            ));
        }

        let radical: u32 = radical_str.parse().unwrap_or_else(|error| {
            fail(&format!(
                "{}:{line}: invalid radical '{radical_str}' for {code_str}: {error}",
                source.display()
            ));
        });
        if !(1..=214).contains(&radical) {
            fail(&format!(
                "{}:{line}: radical must be in 1..=214 for {code_str} (got {radical})",
                source.display()
            ));
        }

        let residual: i32 = residual_str.parse().unwrap_or_else(|error| {
            fail(&format!(
                "{}:{line}: invalid residual stroke count '{residual_str}' for {code_str}: {error}",
                source.display()
            ));
        });
        if !(-999..=999).contains(&residual) {
            fail(&format!(
                "{}:{line}: residual stroke count must fit in the radical bucket for {code_str} (got {residual})",
                source.display()
            ));
        }

        let packed = (radical as i32) * 1000 + residual;
        if packed <= 0 {
            fail(&format!(
                "{}:{line}: packed radical key must be positive for {code_str} (got {packed})",
                source.display()
            ));
        }
        let packed = packed as u32;
        seen_entries.insert((code_u32, packed));
        map.entry(code_u32, &packed.to_string());
    }

    writeln!(
        &mut writer,
        "pub static RADICAL_MAP: ::phf::Map<u32, u32> = {};\n",
        map.build()
    )
    .unwrap_or_else(|error| fail(&format!("failed to write {}: {error}", out_path.display())));

    for (required, expected) in [(0x4E00_u32, 1000_u32), (0x4E2D, 2003)] {
        if !seen_entries.contains(&(required, expected)) {
            fail(&format!(
                "required codepoint U+{required:04X} with radical key {expected} is missing from generated radical data; \
                 regenerate {} from the upstream Unihan dataset",
                source.display()
            ));
        }
    }
}

#[cfg(feature = "collator-zhuyin")]
fn generate_zhuyin_map(data_csv: &Path, out_path: &Path) {
    let mut writer = create_output(out_path);
    let mut rdr = open_csv(data_csv);

    let mut rank_map = Map::new();
    let mut map = Map::new();
    let mut seen_codepoints = BTreeSet::new();
    let source = PathBuf::from(data_csv);

    writeln!(
        &mut writer,
        "// This file is generated by build.rs. Do not edit manually.\n"
    )
    .unwrap_or_else(|error| fail(&format!("failed to write {}: {error}", out_path.display())));

    for (ch, rank) in BOPOMOFO_RANKS {
        rank_map.entry(*ch, &format!("{rank}_u8"));
    }
    writeln!(
        &mut writer,
        "#[allow(dead_code)]\npub static BOPOMOFO_RANK: ::phf::Map<char, u8> = {};\n",
        rank_map.build()
    )
    .unwrap_or_else(|error| fail(&format!("failed to write {}: {error}", out_path.display())));

    for (index, result) in rdr.records().enumerate() {
        let line = index + 2;
        let record = result.unwrap_or_else(|error| {
            fail(&format!(
                "{}:{line}: failed to read CSV row: {error}",
                source.display()
            ));
        });
        if record.len() != 3 {
            fail(&format!(
                "{}:{line}: expected exactly 3 columns (codepoint, zhuyin, char), got {}",
                source.display(),
                record.len()
            ));
        }

        let code_str = &record[0];
        let zhuyin = &record[1];
        let char_field = &record[2];
        let code_u32 = parse_codepoint(code_str, &source, line);

        let mut chars_iter = char_field.chars();
        let character = chars_iter.next().unwrap_or_else(|| {
            fail(&format!(
                "{}:{line}: empty character column for {code_str}",
                source.display()
            ))
        });
        if chars_iter.next().is_some() {
            fail(&format!(
                "{}:{line}: character column must contain exactly one Unicode scalar (got {char_field:?})",
                source.display()
            ));
        }
        if character as u32 != code_u32 {
            fail(&format!(
                "{}:{line}: character {character:?} does not match codepoint {code_str}",
                source.display()
            ));
        }

        let encoded = encode_zhuyin_key(zhuyin, &source, line, code_str);
        seen_codepoints.insert(code_u32);
        map.entry(code_u32, &format!("{encoded}_u128"));
    }

    writeln!(
        &mut writer,
        "pub static ZHUYIN_MAP: ::phf::Map<u32, u128> = {};\n",
        map.build()
    )
    .unwrap_or_else(|error| fail(&format!("failed to write {}: {error}", out_path.display())));

    for required in [0x3007_u32, 0x4E2D, 0x4E00] {
        if !seen_codepoints.contains(&required) {
            fail(&format!(
                "required codepoint U+{required:04X} is missing from generated zhuyin data; \
                 regenerate {} from the bundled pinyin dataset",
                source.display()
            ));
        }
    }
}
