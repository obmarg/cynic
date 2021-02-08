use strsim::hamming;

/// Using Hamming algorithm to guess possible similar fields.
pub fn guess_field<'a>(
    candidates: impl Iterator<Item = &'a str>,
    field_name: &str,
) -> Option<&'a str> {
    candidates.min_by_key(|candidate| hamming(candidate, field_name).unwrap_or(usize::max_value()))
}

pub fn format_guess(guess_field: Option<&str>) -> String {
    match guess_field {
        Some(v) => format!(" Did you mean {}?", v),
        None => "".to_owned(),
    }
}
