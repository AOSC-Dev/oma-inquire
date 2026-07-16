use crate::prompts::prompt::ActionResult;
use crate::validator::ErrorMessage;
use crate::{
    error::InquireResult,
    formatter::BoolFormatter,
    prompts::{
        confirm::config::ConfirmConfig,
        prompt::{ActionState, Prompt},
    },
    ui::ConfirmBackend,
    Confirm,
};

use super::action::ConfirmPromptAction;

pub struct ConfirmPrompt<'a> {
    message: &'a str,
    help_message: Option<&'a str>,
    default: Option<bool>,
    current_value: bool,
    error_message: String,
    error: Option<ErrorMessage>,
    formatter: BoolFormatter<'a>,
    default_value_formatter: BoolFormatter<'a>,
}

impl<'a> From<Confirm<'a>> for ConfirmPrompt<'a> {
    fn from(co: Confirm<'a>) -> Self {
        let current_value = co.default.unwrap_or(false);

        Self {
            message: co.message,
            default: co.default,
            current_value,
            help_message: co.help_message,
            formatter: co.formatter,
            default_value_formatter: co.default_value_formatter,
            error_message: co.error_meesage,
            error: None,
        }
    }
}

impl<'a, Backend> Prompt<Backend> for ConfirmPrompt<'a>
where
    Backend: ConfirmBackend,
{
    type Config = ConfirmConfig;
    type InnerAction = ConfirmPromptAction;
    type Output = bool;

    fn message(&self) -> &str {
        self.message
    }

    fn config(&self) -> &Self::Config {
        &ConfirmConfig {}
    }

    fn format_answer(&self, answer: &bool) -> String {
        (self.formatter)(*answer)
    }

    fn submit(&mut self) -> InquireResult<Option<bool>> {
        Ok(Some(self.current_value))
    }

    fn handle(&mut self, action: ConfirmPromptAction) -> InquireResult<ActionState> {
        let result = match action {
            // 用户输入了答案，立即提交对应的布尔值
            ConfirmPromptAction::DirectSubmit(val) => {
                self.error = None;
                self.current_value = val;
                ActionState::RequestSubmit
            }
            // 按回车直接提交。如果有默认值，以默认值为准
            // 若无，则报错
            ConfirmPromptAction::SubmitDefault => {
                if let Some(default_val) = self.default {
                    self.current_value = default_val;
                    ActionState::RequestSubmit
                } else {
                    self.error = Some(ErrorMessage::Custom(self.error_message.to_string()));
                    ActionState::Render(ActionResult::NeedsRedraw)
                }
            }
            // 退出
            ConfirmPromptAction::PressEscape => ActionState::RequestCancel,
        };

        Ok(result)
    }

    fn render(&self, backend: &mut Backend) -> InquireResult<()> {
        let prompt = &self.message;

        if let Some(err_msg) = &self.error {
            backend.render_error_message(err_msg)?;
        }

        // 格式化括号内默认值的提示，如 "Y/n"
        let default_value_formatter = self.default_value_formatter;
        let default_message = self
            .default
            .as_ref()
            .map(|val| default_value_formatter(*val));

        backend.render_prompt(prompt, default_message.as_deref())?;

        if let Some(message) = self.help_message {
            backend.render_help_message(message)?;
        }

        Ok(())
    }
}
