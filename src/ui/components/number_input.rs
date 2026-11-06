use super::{
    state::{NumberConfig, Validation},
    text_input::{NumericSyntax, TextInput},
};
use gpui::Context;
pub(crate) struct NumberInput;
impl NumberInput {
    pub fn create(
        value: String,
        placeholder: String,
        config: NumberConfig,
        cx: &mut Context<TextInput>,
    ) -> Validation<TextInput> {
        config.parse(&value)?;
        let mut input = TextInput::new(
            value.clone(),
            placeholder,
            NumericSyntax {
                decimal: config.decimals,
                signed: config.negative,
            },
            cx,
        );
        input.configure_number(config, value, cx)?;
        Ok(input)
    }
}
