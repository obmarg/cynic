use darling::util::SpannedValue;
use proc_macro2::Span;

pub trait ExtractString<T> {
    fn extract_spanned_value(&self, ident: String) -> SpannedValue<String>;
    fn extract_spanned(&self) -> Span;
}

impl ExtractString<Option<SpannedValue<String>>> for Option<SpannedValue<String>> {
    fn extract_spanned_value(&self, ident: String) -> SpannedValue<String> {
        self.clone()
            .unwrap_or(SpannedValue::new(ident, Span::call_site()))
    }

    fn extract_spanned(&self) -> Span {
        self.clone().unwrap_or(SpannedValue::default()).span()
    }
}
