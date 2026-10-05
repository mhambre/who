// Hand-written table derived from data/cjk-blocks.txt (Unicode 15.0).
// Re-copy the ranges whenever the vendored file is updated.
who::warn!(
    file("data/cjk-blocks.txt").changed_from("sha256:8e686a76202f986b3b12cef1f8ef951dd80f8620c015d189c0d6c21a90914b20"),
    "data/cjk-blocks.txt changed: regenerate CJK_RANGES and update the baseline hash"
);

const CJK_RANGES: &[(u32, u32)] = &[
    (0x4E00, 0x9FFF),
    (0x3400, 0x4DBF),
    (0x20000, 0x2A6DF),
    (0x2A700, 0x2B739),
    (0x2B740, 0x2B81D),
    (0x2B820, 0x2CEA1),
    (0x2CEB0, 0x2EBE0),
    (0x30000, 0x3134A),
];

fn is_cjk_ideograph(c: char) -> bool {
    let c = c as u32;
    CJK_RANGES.iter().any(|&(lo, hi)| (lo..=hi).contains(&c))
}

fn main() {
    for c in ['漢', 'a', '\u{2EBF0}'] {
        println!("{c:?}: {}", is_cjk_ideograph(c));
    }
}
