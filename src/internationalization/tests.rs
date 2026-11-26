use super::*;
use serde_json::{Value, json};
fn f(locale: &str, value: impl Display, options: Value) -> String {
    Internationalization::format_number(locale, value, &serde_json::from_value(options).unwrap())
        .unwrap()
}
#[test]
fn locale_separators_regional_data_and_currencies() {
    assert_eq!(f("en-US", 1234567.89, json!({})), "1,234,567.89");
    assert_eq!(
        f("fr-FR", 1234567.89, json!({})),
        "1\u{202f}234\u{202f}567,89"
    );
    assert_eq!(f("de-DE", 1234567.89, json!({})), "1.234.567,89");
    assert_eq!(f("en-IN", 1234567.89, json!({})), "12,34,567.89");
    assert_eq!(
        f(
            "en-US",
            1234.5,
            json!({"style":"currency","currency":"EUR"})
        ),
        "€1,234.50"
    );
    assert_eq!(
        f(
            "fr-FR",
            1234.5,
            json!({"style":"currency","currency":"EUR"})
        ),
        "1\u{202f}234,50\u{a0}€"
    );
    assert_eq!(
        f(
            "en-US",
            1234.5,
            json!({"style":"currency","currency":"JPY"})
        ),
        "¥1,235"
    );
    assert_eq!(
        f(
            "fr-FR",
            0.015,
            json!({"style":"percent","maximumFractionDigits":2})
        ),
        "1,5\u{a0}%"
    );
}
#[test]
fn precision_notations_and_decimal_rounding() {
    assert_eq!(
        f("en", 123456, json!({"maximumSignificantDigits":3})),
        "123,000"
    );
    assert_eq!(f("en", 12345, json!({"notation":"scientific"})), "1.235E4");
    assert_eq!(
        f("en", 12345, json!({"notation":"engineering"})),
        "12.345E3"
    );
    assert_eq!(f("en", 1500, json!({"notation":"compact"})), "1.5K");
    assert_eq!(
        f(
            "en",
            1500,
            json!({"notation":"compact","compactDisplay":"long"})
        ),
        "1.5 thousand"
    );
    assert_eq!(
        f("fr-FR", 1500, json!({"notation":"compact"})),
        "1,5\u{a0}k"
    );
    assert_eq!(f("en", 1.005, json!({"maximumFractionDigits":2})), "1.01");
    assert_eq!(f("en", 1.2, json!({"maximumSignificantDigits":21})), "1.2");
    assert_eq!(
        f("en", u64::MAX, json!({"useGrouping":false})),
        "18446744073709551615"
    );
}
#[test]
fn trailing_zeros_and_unit_plurals() {
    assert_eq!(f("en", 1, json!({"minimumFractionDigits":2})), "1.00");
    assert_eq!(
        f(
            "en",
            1,
            json!({"minimumFractionDigits":2,"trailingZeroDisplay":"stripIfInteger"})
        ),
        "1"
    );
    assert_eq!(
        f(
            "en",
            1.5,
            json!({"minimumFractionDigits":2,"trailingZeroDisplay":"stripIfInteger"})
        ),
        "1.50"
    );
    assert_eq!(
        f(
            "en",
            1.0001,
            json!({"minimumFractionDigits":2,"maximumFractionDigits":2,"trailingZeroDisplay":"stripIfInteger","style":"unit","unit":"meter","unitDisplay":"long"})
        ),
        "1 meter"
    );
    assert_eq!(
        f(
            "en",
            1.2,
            json!({"minimumSignificantDigits":4,"trailingZeroDisplay":"stripToMinimum"})
        ),
        "1.200"
    );
    assert_eq!(
        f(
            "en",
            "1.2000",
            json!({"minimumFractionDigits":1,"trailingZeroDisplay":"stripToMinimum"})
        ),
        "1.2"
    );
    assert_eq!(
        f(
            "ar-EG",
            1,
            json!({"minimumFractionDigits":2,"trailingZeroDisplay":"stripIfInteger"})
        ),
        "١"
    );
}
#[test]
fn every_requested_unit_width_and_compounds() {
    let units = [
        "bit",
        "byte",
        "kilobit",
        "kilobyte",
        "megabit",
        "megabyte",
        "gigabit",
        "gigabyte",
        "terabit",
        "terabyte",
        "petabyte",
        "year",
        "month",
        "week",
        "day",
        "hour",
        "minute",
        "second",
        "millisecond",
        "centimeter",
        "meter",
        "kilometer",
        "millimeter",
        "foot",
        "inch",
        "yard",
        "mile",
        "mile-scandinavian",
        "gram",
        "kilogram",
        "ounce",
        "pound",
        "stone",
        "liter",
        "milliliter",
        "gallon",
        "fluid-ounce",
        "acre",
        "hectare",
        "celsius",
        "fahrenheit",
        "degree",
        "percent",
    ];
    for unit in units {
        for lang in ["en-US", "fr-FR"] {
            for width in ["long", "short", "narrow"] {
                assert_ne!(
                    f(
                        lang,
                        12.34,
                        json!({"style":"unit","unit":unit,"unitDisplay":width})
                    ),
                    f(lang, 12.34, json!({})),
                    "{unit} {lang} {width}"
                );
            }
        }
    }
    assert_eq!(
        f(
            "en",
            2.5,
            json!({"style":"unit","unit":"meter-per-second","unitDisplay":"long"})
        ),
        "2.5 meters per second"
    );
    assert_eq!(
        f(
            "en",
            2.5,
            json!({"style":"unit","unit":"kilogram-per-square-meter","unitDisplay":"long"})
        ),
        "2.5 kilograms per square meter"
    );
    assert_eq!(
        f(
            "fr-FR",
            2.5,
            json!({"style":"unit","unit":"kilogram-per-square-meter","unitDisplay":"long"})
        ),
        "2,5\u{a0}kilogrammes par mètre carré"
    );
    assert_eq!(
        f(
            "en",
            2.5,
            json!({"style":"unit","unit":"kilogram-per-square-meter","unitDisplay":"narrow"})
        ),
        "2.5kg/m²"
    );
}
#[test]
fn invalid_inputs_special_values_and_reuse() {
    for options in [
        json!({"style":"currency"}),
        json!({"style":"unit","unit":"unknown"}),
        json!({"style":"unit","unit":"meter-per-second-per-hour"}),
        json!({"minimumFractionDigits":4,"maximumFractionDigits":2}),
        json!({"maximumSignificantDigits":0}),
        json!({"trailingZeroDisplay":"invalid"}),
    ] {
        assert!(
            Internationalization::format_number("en", 1, &serde_json::from_value(options).unwrap())
                .is_err()
        );
    }
    assert!(Internationalization::format_number("", 1, &Default::default()).is_err());
    assert_eq!(f("en", f64::NAN, json!({})), "NaN");
    assert_eq!(f("en", f64::INFINITY, json!({})), "∞");
    assert_eq!(f("en", -0.0, json!({})), "-0");
    assert_eq!(f("zz-ZZ", 1234.5, json!({})), "1,234.5");
    let formatter = Internationalization::number_formatter(
        "fr-FR",
        &serde_json::from_value(json!({"maximumFractionDigits":2})).unwrap(),
    )
    .unwrap();
    assert_eq!(formatter.format(1.5).unwrap(), "1,5");
    assert_eq!(formatter.format(25).unwrap(), "25");
}
