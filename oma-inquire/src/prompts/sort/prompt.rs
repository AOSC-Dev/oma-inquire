use std::fmt::Display;

use crate::{
    error::InquireResult,
    formatter::MultiOptionFormatter,
    list_option::ListOption,
    prompts::{
        prompt::{ActionResult, ActionState, Prompt},
        sort::Sort,
    },
    ui::SortBackend,
    utils::paginate,
    InquireError,
};

use super::{action::SortPromptAction, config::SortConfig};

pub struct SortPrompt<'a, T> {
    message: &'a str,
    config: SortConfig,
    options: Vec<T>,
    string_options: Vec<String>,
    help_message: Option<&'a str>,
    cursor_index: usize,
    formatter: MultiOptionFormatter<'a, T>,
    is_grabbing: bool,
}

impl<'a, T> SortPrompt<'a, T>
where
    T: Display,
{
    pub fn new(so: Sort<'a, T>) -> InquireResult<Self> {
        if so.options.is_empty() {
            return Err(InquireError::InvalidConfiguration(
                "Available options can not be empty".into(),
            ));
        }

        if so.starting_cursor >= so.options.len() {
            return Err(InquireError::InvalidConfiguration(format!(
                "Starting cursor {} is out-of-bounds for length {}",
                so.starting_cursor,
                so.options.len()
            )));
        }

        let string_options = so.options.iter().map(T::to_string).collect();

        Ok(Self {
            message: so.message,
            config: (&so).into(),
            options: so.options,
            string_options,
            help_message: so.help_message,
            cursor_index: so.starting_cursor,
            formatter: so.formatter,
            is_grabbing: false,
        })
    }

    fn move_cursor_up(&mut self, qty: usize, wrap: bool) -> ActionResult {
        let new_position = if wrap {
            let after_wrap = qty.saturating_sub(self.cursor_index);
            self.cursor_index
                .checked_sub(qty)
                .unwrap_or_else(|| self.options.len().saturating_sub(after_wrap))
        } else {
            self.cursor_index.saturating_sub(qty)
        };

        self.update_cursor_position(new_position)
    }

    fn move_cursor_down(&mut self, qty: usize, wrap: bool) -> ActionResult {
        let mut new_position = self.cursor_index.saturating_add(qty);

        if new_position >= self.options.len() {
            new_position = if self.options.is_empty() {
                0
            } else if wrap {
                new_position % self.options.len()
            } else {
                self.options.len().saturating_sub(1)
            }
        }

        self.update_cursor_position(new_position)
    }

    fn update_cursor_position(&mut self, new_position: usize) -> ActionResult {
        if new_position != self.cursor_index {
            self.cursor_index = new_position;
            ActionResult::NeedsRedraw
        } else {
            ActionResult::Clean
        }
    }

    /// 将当前高亮选项向上拖拽一个身位
    fn drag_item_up(&mut self) -> ActionResult {
        if self.cursor_index > 0 {
            let target_index = self.cursor_index - 1;
            self.options.swap(self.cursor_index, target_index);
            self.string_options.swap(self.cursor_index, target_index);
            self.cursor_index = target_index; // 光标跟随着往前移
            ActionResult::NeedsRedraw
        } else {
            ActionResult::Clean
        }
    }

    /// 将当前高亮选项向下拖拽一个身位
    fn drag_item_down(&mut self) -> ActionResult {
        if self.cursor_index < self.options.len().saturating_sub(1) {
            let target_index = self.cursor_index + 1;
            self.options.swap(self.cursor_index, target_index);
            self.string_options.swap(self.cursor_index, target_index);
            self.cursor_index = target_index; // 光标跟随着往后移
            ActionResult::NeedsRedraw
        } else {
            ActionResult::Clean
        }
    }

    /// 排序组件完成后直接获取完整的所有选项排序成果
    fn get_final_answer(&mut self) -> Vec<ListOption<T>> {
        let mut answer = vec![];
        let drained = std::mem::take(&mut self.options);
        for (index, value) in drained.into_iter().enumerate() {
            answer.push(ListOption::new(index, value));
        }
        answer
    }
}

impl<'a, Backend, T> Prompt<Backend> for SortPrompt<'a, T>
where
    Backend: SortBackend,
    T: Display,
{
    type Config = SortConfig;
    type InnerAction = SortPromptAction;
    type Output = Vec<ListOption<T>>;

    fn message(&self) -> &str {
        self.message
    }

    fn config(&self) -> &SortConfig {
        &self.config
    }

    fn format_answer(&self, answer: &Vec<ListOption<T>>) -> String {
        let refs: Vec<ListOption<&T>> = answer.iter().map(ListOption::as_ref).collect();
        (self.formatter)(&refs)
    }

    fn submit(&mut self) -> InquireResult<Option<Vec<ListOption<T>>>> {
        Ok(Some(self.get_final_answer()))
    }

    fn handle(&mut self, action: SortPromptAction) -> InquireResult<ActionState> {
        let result = match action {
            SortPromptAction::ToggleOption => {
                self.is_grabbing = !self.is_grabbing;
                ActionState::Render(ActionResult::NeedsRedraw)
            }
            SortPromptAction::MoveUp => ActionState::Render(if self.is_grabbing {
                self.drag_item_up()
            } else {
                self.move_cursor_up(1, true)
            }),
            SortPromptAction::MoveDown => ActionState::Render(if self.is_grabbing {
                self.drag_item_down()
            } else {
                self.move_cursor_down(1, true)
            }),
            SortPromptAction::PageUp => {
                ActionState::Render(self.move_cursor_up(self.config.page_size, false))
            }
            SortPromptAction::PageDown => {
                ActionState::Render(self.move_cursor_down(self.config.page_size, false))
            }
            SortPromptAction::MoveToStart => {
                ActionState::Render(self.move_cursor_up(usize::MAX, false))
            }
            SortPromptAction::MoveToEnd => {
                ActionState::Render(self.move_cursor_down(usize::MAX, false))
            }
            SortPromptAction::PressEscape => {
                if self.is_grabbing {
                    self.is_grabbing = false;
                    ActionState::Render(ActionResult::NeedsRedraw)
                } else {
                    ActionState::RequestCancel
                }
            }
            SortPromptAction::Submit => ActionState::RequestSubmit,
        };

        Ok(result)
    }

    fn render(&self, backend: &mut Backend) -> InquireResult<()> {
        let prompt = &self.message;

        backend.render_sort_prompt(*prompt)?;

        let choices = self
            .options
            .iter()
            .enumerate()
            .map(|(i, opt)| ListOption::new(i, opt))
            .collect::<Vec<ListOption<&T>>>();

        let page = paginate(self.config.page_size, &choices, Some(self.cursor_index));

        backend.render_sort_options(page, self.cursor_index, self.is_grabbing)?;

        if let Some(help_message) = self.help_message {
            backend.render_help_message(help_message)?;
        }

        Ok(())
    }
}
