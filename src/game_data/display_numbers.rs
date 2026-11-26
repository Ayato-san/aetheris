//! Common display formats. Prepared formatters are reused per rendering/worker
//! thread; at most 64 locale/format pairs are retained. Inputs remain exact.
use crate::internationalization::{Internationalization, NumberFormatOptions, NumberFormatter};
use std::{cell::RefCell, collections::BTreeMap, fmt::Display};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Presentation {
    Integer,
    SignedInteger,
    Decimal(u8),
    Fixed(u8),
    Percent(u8),
    SignedPercent(u8),
    Metres,
    SignedMetres,
}
thread_local! {
    static FORMATTERS: RefCell<BTreeMap<(String, Presentation), NumberFormatter>> = const { RefCell::new(BTreeMap::new()) };
}
/// Presentation fallback preserves the original value when source text or its
/// locale is invalid, so a malformed catalogue cannot hide the displayed data.
pub fn number(language: &str, value: impl Display, format: Presentation) -> String {
    let value = value.to_string();
    FORMATTERS.with(|cache| {
        let mut cache = cache.borrow_mut();
        let key = (language.to_owned(), format);
        if !cache.contains_key(&key) {
            let mut options = NumberFormatOptions::default();
            let digits = match format {
                Presentation::Integer | Presentation::SignedInteger => 0,
                Presentation::Decimal(n)
                | Presentation::Fixed(n)
                | Presentation::Percent(n)
                | Presentation::SignedPercent(n) => n,
                Presentation::Metres | Presentation::SignedMetres => 2,
            };
            options.digits.maximum_fraction_digits = Some(digits.into());
            if matches!(format, Presentation::Fixed(_)) {
                options.digits.minimum_fraction_digits = Some(digits.into());
            }
            if matches!(
                format,
                Presentation::SignedInteger
                    | Presentation::SignedPercent(_)
                    | Presentation::SignedMetres
            ) {
                options.sign_display = Some("always".into());
            }
            if matches!(
                format,
                Presentation::Percent(_) | Presentation::SignedPercent(_)
            ) {
                options.style = Some("unit".into());
                options.unit = Some("percent".into());
                options.unit_display = Some("short".into());
            }
            if matches!(format, Presentation::Metres | Presentation::SignedMetres) {
                options.style = Some("unit".into());
                options.unit = Some("meter".into());
                options.unit_display = Some("narrow".into());
            }
            let Ok(formatter) = Internationalization::number_formatter(language, &options) else {
                return value;
            };
            if cache.len() >= 64 {
                cache.clear();
            }
            cache.insert(key.clone(), formatter);
        }
        cache[&key].format(&value).unwrap_or(value)
    })
}

/// Exact decimal representation of a fixed-point integer. This is scaling,
/// not presentation; no floating-point rounding or 64-bit precision loss occurs.
pub fn hundredths(value: i64) -> String {
    let magnitude = value.unsigned_abs();
    format!(
        "{}{}.{:02}",
        if value < 0 { "-" } else { "" },
        magnitude / 100,
        magnitude % 100
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn common_formats_reuse_locales_and_preserve_fixed_point_precision() {
        assert_eq!(
            number("fr-FR", hundredths(150), Presentation::Percent(2)),
            "1,5 %"
        );
        assert_eq!(
            number("en-US", hundredths(150), Presentation::SignedPercent(2)),
            "+1.5%"
        );
        assert_eq!(
            number("fr-FR", 12345, Presentation::Integer),
            "12\u{202f}345"
        );
        assert_eq!(number("en-US", 12345, Presentation::Integer), "12,345");
        assert_eq!(number("fr-FR", 1.25, Presentation::Metres), "1,25m");
        assert_eq!(number("fr-FR", 1.5, Presentation::Fixed(2)), "1,50");
        assert_eq!(hundredths(i64::MIN), "-92233720368547758.08");
        assert_eq!(
            number("fr-FR", "not a number", Presentation::Integer),
            "not a number"
        );
    }
}
