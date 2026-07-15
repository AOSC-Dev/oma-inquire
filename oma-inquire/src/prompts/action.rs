//! Definitions for the broad Action type which encompasses
//! the directives for prompts.

use std::fmt::Debug;

use crate::ui::{Key, KeyModifiers};

/// Top-level type to describe the directives a prompt
/// receives.
///
/// Each prompt should implement its own custom InnerAction type
/// which is parsed and stored in the Inner variant, if applicable,
/// on the normal execution flow of a prompt.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Action<I>
where
    I: Copy + Clone + PartialEq + Eq,
{
    /// Submits the current prompt answer, finishing the prompt if valid.
    Submit,
    /// Cancels the prompt execution with a graceful shutdown.
    Cancel,
    /// Interrupts the prompt execution without a graceful shutdown.
    Interrupt,
    /// Specialized actions according to the prompt type.
    Inner(I),
}

impl<I> Action<I>
where
    I: Copy + Clone + PartialEq + Eq,
{
    /// Derives a prompt action from a Key event.
    pub fn from_key<C>(key: Key, config: &C) -> Option<Action<I>>
    where
        I: InnerAction<Config = C>,
    {
        match key {
            Key::Enter
            | Key::Char('\n', KeyModifiers::NONE)
            | Key::Char('j', KeyModifiers::CONTROL)
                if I::ESCAPE_POLICY == EscapePolicy::CancelOnEscape =>
            {
                Some(Action::Submit)
            }
            Key::Escape if I::ESCAPE_POLICY == EscapePolicy::SubmitOnEscape => Some(Action::Submit),
            Key::Escape if I::ESCAPE_POLICY == EscapePolicy::CancelOnEscape => Some(Action::Cancel),
            Key::Char('c', KeyModifiers::CONTROL) => Some(Action::Interrupt),
            key => I::from_key(key, config).map(Action::Inner),
        }
    }
}

/// Defers and configures the core submit/cancel control flow behavior for prompt inputs.
///
/// This policy decouples individual components from the global key mapper. Instead of hardcoding
/// specific component behaviors (e.g., checking if an action belongs to a multi-select prompt),
/// components advertise their structural key handling strategy via this enum.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EscapePolicy {
    /// Esc 取消组件
    CancelOnEscape,
    /// Esc 作为确认并提交
    SubmitOnEscape,
    /// 把 Esc 忽略或交给自定义配置
    IgnoreEscape,
}

/// InnerActions are specialized prompt actions.
///
/// They must provide an implementation to optionally derive an action
/// from a key event.
pub trait InnerAction
where
    Self: Sized + Copy + Clone + PartialEq + Eq,
{
    /// 默认情况下，cancel 为退出，enter 为 submit
    const ESCAPE_POLICY: EscapePolicy = EscapePolicy::CancelOnEscape;

    /// Configuration type for the prompt.
    ///
    /// This is used to derive the action from a key event.
    type Config;

    /// Derives a prompt action from a Key event and the prompt configuration.
    fn from_key(key: Key, config: &Self::Config) -> Option<Self>
    where
        Self: Sized;
}

#[cfg(test)]
mod test {
    use crate::{
        ui::{Key, KeyModifiers},
        Action, InnerAction,
    };

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub enum MockInnerAction {
        Action(Key),
    }

    impl InnerAction for MockInnerAction {
        type Config = ();

        fn from_key(key: Key, _config: &()) -> Option<Self>
        where
            Self: Sized,
        {
            Some(Self::Action(key))
        }
    }

    #[test]
    fn standard_keybindings_for_submit() {
        let key = Key::Enter;
        assert_eq!(
            Some(Action::<MockInnerAction>::Submit),
            Action::from_key(key, &())
        );
    }

    #[test]
    fn standard_keybindings_for_cancel() {
        let key = Key::Escape;
        assert_eq!(
            Some(Action::<MockInnerAction>::Cancel),
            Action::from_key(key, &())
        );
    }

    #[test]
    fn ctrl_c_results_in_interrupt_action() {
        let key = Key::Char('c', KeyModifiers::CONTROL);
        assert_eq!(
            Some(Action::<MockInnerAction>::Interrupt),
            Action::from_key(key, &())
        );
    }

    #[test]
    fn generic_keys_are_passed_down_to_inner_action() {
        assert_eq!(
            Some(Action::<MockInnerAction>::Inner(MockInnerAction::Action(
                Key::Char('a', KeyModifiers::NONE)
            ))),
            Action::from_key(Key::Char('a', KeyModifiers::NONE), &())
        );
        assert_eq!(
            Some(Action::<MockInnerAction>::Inner(MockInnerAction::Action(
                Key::Home
            ))),
            Action::from_key(Key::Home, &())
        );
        assert_eq!(
            Some(Action::<MockInnerAction>::Inner(MockInnerAction::Action(
                Key::PageDown(KeyModifiers::NONE)
            ))),
            Action::from_key(Key::PageDown(KeyModifiers::NONE), &())
        );
    }

    #[test]
    fn emacs_control_keybindings() {
        assert_eq!(
            Some(Action::<MockInnerAction>::Submit),
            Action::from_key(Key::Char('j', KeyModifiers::CONTROL), &())
        );
        assert_eq!(
            Some(Action::<MockInnerAction>::Cancel),
            Action::from_key(Key::Char('g', KeyModifiers::CONTROL), &())
        );
    }
}
