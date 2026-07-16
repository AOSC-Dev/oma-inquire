use crate::{
    prompts::confirm::config::ConfirmConfig,
    ui::{Key, KeyModifiers},
    EscapePolicy, InnerAction,
};

/// Confrim 组件
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConfirmPromptAction {
    /// 用户输入了答案（例如默认的 y/n 或 Y/N），会立即提交
    DirectSubmit(bool),
    /// 用户按了回车（Enter），使用当前的默认值提交
    /// 若没有默认值，则会报错提示用户输入答案
    SubmitDefault,
    /// Escape 事件
    PressEscape,
}

impl InnerAction for ConfirmPromptAction {
    type Config = ConfirmConfig;

    const ESCAPE_POLICY: EscapePolicy = EscapePolicy::IgnoreEscape;

    fn from_key(key: Key, _config: &Self::Config) -> Option<Self> {
        match key {
            Key::Char('y', KeyModifiers::NONE)
            | Key::Char('Y', KeyModifiers::NONE)
            | Key::Char('Y', KeyModifiers::SHIFT) => Some(Self::DirectSubmit(true)),
            Key::Char('n', KeyModifiers::NONE)
            | Key::Char('N', KeyModifiers::NONE)
            | Key::Char('N', KeyModifiers::SHIFT) => Some(Self::DirectSubmit(false)),
            Key::Enter
            | Key::Char('\n', KeyModifiers::NONE)
            | Key::Char('j', KeyModifiers::CONTROL) => Some(Self::SubmitDefault),
            Key::Escape => Some(Self::PressEscape),
            _ => None,
        }
    }
}
