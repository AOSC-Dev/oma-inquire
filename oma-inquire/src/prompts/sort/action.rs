use crate::{
    ui::{Key, KeyModifiers},
    EscapePolicy, InnerAction,
};

use super::config::SortConfig;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SortPromptAction {
    /// Moves the cursor to the option above.
    /// If the current option is selected, this will drag/reorder it up.
    MoveUp,
    /// Moves the cursor to the option below.
    /// If the current option is selected, this will drag/reorder it down.
    MoveDown,
    /// Moves the cursor to the page above.
    PageUp,
    /// Moves the cursor to the page below.
    PageDown,
    /// Moves the cursor to the start of the list.
    MoveToStart,
    /// Moves the cursor to the end of the list.
    MoveToEnd,
    /// Toggles the selected state (active/inactive) of the current highlighted item.
    ToggleOption,
    /// Press Escape
    PressEscape,
    /// Submit order
    Submit,
}

impl InnerAction for SortPromptAction {
    const ESCAPE_POLICY: EscapePolicy = EscapePolicy::IgnoreEscape;
    type Config = SortConfig;

    fn from_key(key: Key, config: &SortConfig) -> Option<Self> {
        if config.vim_mode {
            let action = match key {
                Key::Char('k', KeyModifiers::NONE) => Some(Self::MoveUp),
                Key::Char('j', KeyModifiers::NONE) => Some(Self::MoveDown),
                _ => None,
            };

            if action.is_some() {
                return action;
            }
        }

        let action = match key {
            // 向上移动（普通光标移动，或在已选中状态下向上拖拽排序）
            Key::Up(KeyModifiers::NONE) | Key::Char('p', KeyModifiers::CONTROL) => Self::MoveUp,

            // 向下移动（普通光标移动，或在已选中状态下向下拖拽排序）
            Key::Down(KeyModifiers::NONE) | Key::Char('n', KeyModifiers::CONTROL) => Self::MoveDown,

            // 翻页与快速跳转
            Key::PageUp(_) => Self::PageUp,
            Key::Home => Self::MoveToStart,
            Key::PageDown(_) => Self::PageDown,
            Key::End => Self::MoveToEnd,

            // 空格键：选中/取消选中
            Key::Char(' ', KeyModifiers::NONE) | Key::Toggle => Self::ToggleOption,
            Key::Escape => Self::PressEscape,
            Key::Enter
            | Key::Char('\n', KeyModifiers::NONE)
            | Key::Char('j', KeyModifiers::CONTROL) => Self::Submit,
            _ => return None,
        };

        Some(action)
    }
}
