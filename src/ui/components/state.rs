//! Pure UI contracts. No domain rules or persistence belong here.
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug)]
pub(crate) struct NumericSyntax {
    pub decimal: bool,
    pub signed: bool,
}
impl NumericSyntax {
    pub const PIXELS: Self = Self {
        decimal: false,
        signed: false,
    };
    pub fn accepts(self, text: &str) -> bool {
        let mut separator = false;
        text.chars().enumerate().all(|(i, c)| match c {
            '0'..='9' => true,
            '-' => self.signed && i == 0,
            '.' | ',' if self.decimal && !separator => {
                separator = true;
                true
            }
            _ => false,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Invalid(pub String);
impl From<&str> for Invalid {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}
pub(crate) type Validation<T> = Result<T, Invalid>;

#[derive(Clone, Debug)]
pub(crate) struct FieldState {
    pub reference: String,
    pub draft: String,
    pub error: Option<String>,
    pub conflict: bool,
}
impl FieldState {
    pub fn new(value: String) -> Self {
        Self {
            reference: value.clone(),
            draft: value,
            error: None,
            conflict: false,
        }
    }
    pub fn dirty(&self) -> bool {
        self.draft != self.reference
    }
    pub fn sync(&mut self, value: String) {
        if self.reference == value {
            return;
        }
        if self.dirty() {
            self.conflict = self.draft != value;
        } else {
            self.draft = value.clone();
        }
        self.reference = value;
    }
    pub fn reset(&mut self, value: String) {
        *self = Self::new(value);
    }
    pub fn cancel(&mut self) -> bool {
        let consumed = self.dirty() || self.error.is_some() || self.conflict;
        self.reset(self.reference.clone());
        consumed
    }
    pub fn accept(&mut self, value: String) -> bool {
        let changed = self.reference != value;
        self.reset(value);
        changed
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NumberConfig {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub decimals: bool,
    pub negative: bool,
    pub allow_empty: bool,
}
impl NumberConfig {
    pub fn validate(self) -> Validation<()> {
        if self.min.is_some_and(|v| !v.is_finite())
            || self.max.is_some_and(|v| !v.is_finite())
            || matches!((self.min, self.max), (Some(a), Some(b)) if a > b)
        {
            return Err("Bornes numériques invalides".into());
        }
        Ok(())
    }
    pub fn normalize(self, text: &str) -> Validation<String> {
        let value = self.parse(text)?;
        Ok(match value {
            None => String::new(),
            Some(value) if self.decimals => value.to_string().replace('.', ","),
            Some(_) => text.to_owned(),
        })
    }
    pub fn parse(self, text: &str) -> Validation<Option<f64>> {
        self.validate()?;
        if text.is_empty() && self.allow_empty {
            return Ok(None);
        }
        let syntax = NumericSyntax {
            decimal: self.decimals,
            signed: self.negative,
        };
        if !syntax.accepts(text) {
            return Err("Format numérique invalide".into());
        }
        let value: f64 = text
            .replace(',', ".")
            .parse()
            .map_err(|_| Invalid::from("Nombre requis"))?;
        if !value.is_finite()
            || (!self.negative && value < 0.)
            || self.min.is_some_and(|v| value < v)
            || self.max.is_some_and(|v| value > v)
        {
            return Err("Nombre hors limites".into());
        }
        Ok(Some(value))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct OptionItem {
    pub id: String,
    pub label: String,
    pub disabled: bool,
}
impl OptionItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Selection {
    pub options: Vec<OptionItem>,
    pub selected: BTreeSet<String>,
    pub min: usize,
    pub max: usize,
}
impl Selection {
    pub fn new(
        options: Vec<OptionItem>,
        selected: BTreeSet<String>,
        min: usize,
        max: usize,
    ) -> Validation<Self> {
        let next = Self {
            options,
            selected,
            min,
            max,
        };
        let ids: BTreeSet<_> = next.options.iter().map(|o| &o.id).collect();
        if min > max
            || max == 0
            || ids.len() != next.options.len()
            || next.selected.len() < min
            || next.selected.len() > max
            || !next.selected.iter().all(|id| ids.contains(id))
        {
            return Err("Options ou sélection incompatibles".into());
        }
        Ok(next)
    }
    pub fn candidate(&self, id: &str) -> Validation<BTreeSet<String>> {
        let item = self
            .options
            .iter()
            .find(|o| o.id == id)
            .ok_or(Invalid::from("Option inconnue"))?;
        if item.disabled {
            return Err("Option indisponible".into());
        }
        let mut next = self.selected.clone();
        if next.contains(id) {
            if next.len() > self.min {
                next.remove(id);
            }
        } else {
            if self.max == 1 {
                next.clear();
            }
            if next.len() >= self.max {
                return Err("Nombre maximal de choix atteint".into());
            }
            next.insert(id.into());
        }
        Ok(next)
    }
    /// Configuration and selection are replaced together, or neither is changed.
    pub fn configure(
        &mut self,
        options: Vec<OptionItem>,
        selected: BTreeSet<String>,
        min: usize,
        max: usize,
    ) -> Validation<()> {
        *self = Self::new(options, selected, min, max)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SliderState {
    pub min: f64,
    pub max: f64,
    pub value: f64,
    pub steps: Vec<f64>,
    pub snap: bool,
    pub keyboard_increment: f64,
}
impl SliderState {
    pub fn validate(&self) -> Validation<()> {
        if !self.min.is_finite()
            || !self.max.is_finite()
            || self.min >= self.max
            || !self.value.is_finite()
            || self.value < self.min
            || self.value > self.max
            || !self.keyboard_increment.is_finite()
            || self.keyboard_increment <= 0.
            || (!self.steps.is_empty()
                && (self.steps.first() != Some(&self.min)
                    || self.steps.last() != Some(&self.max)
                    || self.steps.iter().any(|v| !v.is_finite())
                    || self.steps.windows(2).any(|w| w[0] >= w[1])))
            || (self.snap && !self.steps.is_empty() && !self.steps.contains(&self.value))
        {
            return Err("Configuration du curseur invalide".into());
        }
        Ok(())
    }
    pub fn configure(&mut self, next: Self) -> Validation<()> {
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn nearest(&self, value: f64) -> f64 {
        let value = value.clamp(self.min, self.max);
        if self.snap && !self.steps.is_empty() {
            *self
                .steps
                .iter()
                .min_by(|a, b| {
                    ((**a - value).abs())
                        .total_cmp(&(**b - value).abs())
                        .then_with(|| b.total_cmp(a))
                })
                .unwrap()
        } else {
            value
        }
    }
    pub fn arrow(&self, forward: bool) -> f64 {
        if self.snap && !self.steps.is_empty() {
            if forward {
                self.steps
                    .iter()
                    .copied()
                    .find(|v| *v > self.value)
                    .unwrap_or(self.max)
            } else {
                self.steps
                    .iter()
                    .rev()
                    .copied()
                    .find(|v| *v < self.value)
                    .unwrap_or(self.min)
            }
        } else {
            (self.value
                + if forward {
                    self.keyboard_increment
                } else {
                    -self.keyboard_increment
                })
            .clamp(self.min, self.max)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reference_updates_preserve_drafts_and_escape_uses_latest_reference() {
        let mut field = FieldState::new("1".into());
        field.draft = "12".into();
        field.sync("2".into());
        assert_eq!(field.draft, "12");
        assert!(field.conflict);
        assert!(field.cancel());
        assert_eq!(field.draft, "2");
        assert!(!field.cancel());
        assert!(!field.accept("2".into()));
        assert!(field.accept("3".into()));
    }
    #[test]
    fn numeric_validation_preserves_incomplete_drafts_but_rejects_invalid_values() {
        let config = NumberConfig {
            decimals: true,
            negative: true,
            min: Some(-2.),
            max: Some(2.),
            ..Default::default()
        };
        assert_eq!(config.parse("1,"), Ok(Some(1.)));
        for text in ["-", "", "3", "12abc", "1e0", "NaN"] {
            assert!(config.parse(text).is_err(), "{text}");
        }
        assert_eq!(
            NumberConfig {
                allow_empty: true,
                ..config
            }
            .parse(""),
            Ok(None)
        );
    }
    #[test]
    fn selection_configuration_is_atomic_and_uses_ids() {
        let options = vec![OptionItem::new("a", "Same"), OptionItem::new("b", "Same")];
        let mut selection =
            Selection::new(options.clone(), BTreeSet::from(["a".into()]), 1, 1).unwrap();
        assert_eq!(selection.candidate("a").unwrap(), selection.selected);
        assert_eq!(
            selection.candidate("b").unwrap(),
            BTreeSet::from(["b".into()])
        );
        assert!(
            selection
                .configure(vec![options[1].clone()], selection.selected.clone(), 1, 1)
                .is_err()
        );
        assert_eq!(selection.options.len(), 2);
    }
    #[test]
    fn irregular_steps_snap_up_on_ties_and_configuration_is_atomic() {
        let mut slider = SliderState {
            min: 0.,
            max: 10.,
            value: 3.,
            steps: vec![0., 3., 10.],
            snap: true,
            keyboard_increment: 1.,
        };
        assert_eq!(slider.nearest(6.5), 10.);
        assert_eq!(slider.arrow(true), 10.);
        let mut invalid = slider.clone();
        invalid.value = 4.;
        assert!(slider.configure(invalid).is_err());
        assert_eq!(slider.value, 3.);
    }
}
