//! String operations shared by native INI readers and external components.

/// A `strncpy` cut to `max_bytes` source bytes. The store widens each source
/// byte to one character (`util::native_string::widen_bytes`), so the cut
/// counts characters, not UTF-8 bytes; test text outside that byte domain has
/// no native identity and is cut the same way.
pub fn truncate_native_bytes(value: &str, max_bytes: usize) -> &str {
    value
        .char_indices()
        .nth(max_bytes)
        .map_or(value, |(end, _)| &value[..end])
}
