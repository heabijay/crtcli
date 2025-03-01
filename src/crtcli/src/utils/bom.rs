pub const BOM_CHAR: &str = "\u{FEFF}";
pub const BOM_CHAR_BYTES: &[u8] = BOM_CHAR.as_bytes();

pub fn is_bom(s: &[u8]) -> bool {
    s.starts_with(BOM_CHAR_BYTES)
}

pub fn trim_bom(s: &[u8]) -> &[u8] {
    match is_bom(s) {
        true => &s[BOM_CHAR_BYTES.len()..s.len()],
        false => s,
    }
}
