//! Shared number presentation using embedded CLDR data, independent of GPUI.
//!
//! Options deserialize from the same camelCase names as `Intl.NumberFormat`.
//! Integer and decimal-string inputs retain their precision; no game-specific
//! scaling or conversion of currencies/measurement units occurs here.
use formatjs_intl_numberformat::{NumberFormat, registry::Registry};
use std::{collections::BTreeMap, fmt::Display, sync::OnceLock};

pub use formatjs_intl_numberformat::{Error as FormatError, Options as NumberFormatOptions};

/// Stateless entry point for the application's internationalization services.
pub struct Internationalization;
impl Internationalization {
    /// Format a number using an explicit BCP-47 language and Intl-style options.
    ///
    /// Percent style accepts a ratio (`0.015` → `1.5%`); unit `percent` accepts
    /// a percentage (`1.5` → `1.5%`). A well-formed unsupported locale falls back
    /// to English, deterministically, rather than consulting the OS language.
    pub fn format_number(
        language: &str,
        value: impl Display,
        options: &NumberFormatOptions,
    ) -> Result<String, FormatError> {
        Self::number_formatter(language, options)?.format(value)
    }

    /// Prepare once for repeated formatting, for example when rendering a list.
    pub fn number_formatter(
        language: &str,
        options: &NumberFormatOptions,
    ) -> Result<NumberFormatter, FormatError> {
        static REGISTRY: OnceLock<Registry> = OnceLock::new();
        let registry = REGISTRY.get_or_init(Registry::with_compiled_data);
        let requested = formatjs_intl_getcanonicallocales::get_canonical_locales(&[language])
            .map_err(|_| FormatError::InvalidLocale)?;
        let requested = if registry.supported_locales(&requested).is_empty() {
            vec!["en".into()]
        } else {
            requested
        };
        let mut options = options.clone();
        // square-meter is a CLDR extension to ECMA-402's sanctioned unit list.
        // Format the numerator normally, then apply CLDR's per-square-meter
        // pattern, including localized words, spaces and unit widths.
        let square_meter = if options.style.as_deref() == Some("unit") {
            if let Some((numerator, "square-meter")) =
                options.unit.as_deref().and_then(|u| u.split_once("-per-"))
            {
                let numerator = numerator.to_owned();
                options.unit = Some(numerator);
                true
            } else {
                false
            }
        } else {
            false
        };
        let inner = registry.create(&requested, &options)?;
        let pattern = square_meter.then(|| {
            square_meter_pattern(
                &inner.resolved_options().locale,
                options.unit_display.as_deref().unwrap_or("short"),
            )
        });
        Ok(NumberFormatter {
            inner,
            square_meter_pattern: pattern,
        })
    }
}

/// Reusable immutable formatter; construction validates the locale and options.
pub struct NumberFormatter {
    inner: NumberFormat,
    square_meter_pattern: Option<&'static str>,
}
impl NumberFormatter {
    pub fn format(&self, value: impl Display) -> Result<String, FormatError> {
        let value = value.to_string();
        // Rust and ECMA-402 use different spellings of floating-point infinity.
        let value = match value.as_str() {
            "inf" => "Infinity",
            "-inf" => "-Infinity",
            _ => &value,
        };
        let number = self.inner.format(value)?;
        Ok(match self.square_meter_pattern {
            Some(pattern) => pattern.replace("{0}", &number),
            None => number,
        })
    }
}

#[derive(serde::Deserialize)]
struct SquareMeterPatterns {
    patterns: Vec<[String; 3]>,
    locales: BTreeMap<String, usize>,
}
fn square_meter_pattern(language: &str, width: &str) -> &'static str {
    static DATA: OnceLock<SquareMeterPatterns> = OnceLock::new();
    let data = DATA.get_or_init(|| {
        serde_json::from_str(include_str!("internationalization/square_meter.json"))
            .expect("checked-in CLDR square-meter patterns")
    });
    let mut tag = language.to_ascii_lowercase();
    let index = loop {
        if let Some(index) = data.locales.get(&tag) {
            break *index;
        }
        if let Some(end) = tag.rfind('-') {
            tag.truncate(end);
        } else {
            break data.locales["en"];
        }
    };
    &data.patterns[index][match width {
        "long" => 0,
        "narrow" => 2,
        _ => 1,
    }]
}

#[cfg(test)]
#[path = "internationalization/tests.rs"]
mod tests;
