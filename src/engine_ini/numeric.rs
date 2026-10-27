//! Numeric draft syntax shared by keyboard, paste and composition input.
#[derive(Clone, Copy, Debug)]
pub struct NumericInput {
    pub decimal: bool,
    pub signed: bool,
}
impl NumericInput {
    pub const PIXELS: Self = Self {
        decimal: false,
        signed: false,
    };
    /// Empty and incomplete numbers are allowed while editing; final values
    /// still go through the setting's semantic validation before saving.
    pub fn accepts(self, draft: &str) -> bool {
        let mut separator = false;
        draft.chars().enumerate().all(|(index, ch)| match ch {
            '0'..='9' => true,
            '-' => self.signed && index == 0,
            '.' | ',' if self.decimal && !separator => {
                separator = true;
                true
            }
            _ => false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn integers_reject_text_decimal_and_negative_pixels() {
        let input = NumericInput::PIXELS;
        for value in ["", "0", "1", "99999999999999999999"] {
            assert!(input.accepts(value), "{value}");
        }
        for value in [
            "abc", "12px", "1.5", "1,5", "-1", " 12", "１２", "1e3", "1\n2",
        ] {
            assert!(!input.accepts(value), "{value}");
        }
    }
    #[test]
    fn decimal_drafts_accept_one_separator_and_only_a_leading_allowed_sign() {
        let signed = NumericInput {
            decimal: true,
            signed: true,
        };
        for value in ["", "-", ".", ",", "-0,5", "-1", "1.25", ".5", "1."] {
            assert!(signed.accepts(value), "{value}");
        }
        for value in ["1.2.3", "1,2.3", "--1", "1-2", "NaN", "inf", "10 ms", "2x"] {
            assert!(!signed.accepts(value), "{value}");
        }
        let positive = NumericInput {
            decimal: true,
            signed: false,
        };
        assert!(positive.accepts("1,5"));
        assert!(!positive.accepts("-1"));
    }
}
