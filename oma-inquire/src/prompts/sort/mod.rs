//! Sort prompt to reorder a list of options.

mod action;
mod config;
mod prompt;
#[cfg(test)]
#[cfg(feature = "crossterm")]
mod test;

use std::fmt::Display;

use crate::{
    config::get_configuration,
    error::{InquireError, InquireResult},
    formatter::MultiOptionFormatter,
    list_option::ListOption,
    prompts::prompt::Prompt,
    terminal::get_default_terminal,
    ui::{Backend, RenderConfig, SortBackend},
};

use self::prompt::SortPrompt;

/// Prompt designed for sorting/re-ordering a list of items.
///
/// The user moves the cursor via ↑/↓, presses Space to select/highlight an item,
/// and then moves that item up/down using ↑/↓ or j/k (Vim style) to change its order.
/// Pressing Enter submits the final sorted list.
///
/// # Example
///
/// ```no_run
/// use inquire::Sort;
///
/// let options = vec!["Apple", "Banana", "Cherry"];
/// let sorted = Sort::new("Order your favorite fruits:", options)
///     .prompt()
///     .unwrap();
/// ```
#[derive(Clone)]
pub struct Sort<'a, T> {
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Options/Items to be sorted.
    pub options: Vec<T>,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Page size of the options displayed to the user.
    pub page_size: usize,

    /// Whether vim mode is enabled.
    pub vim_mode: bool,

    /// Starting cursor index.
    pub starting_cursor: usize,

    /// Function that formats the sorted list for final display.
    pub formatter: MultiOptionFormatter<'a, T>,

    /// RenderConfig to apply to the rendered interface.
    pub render_config: RenderConfig<'a>,
}

impl<'a, T> Sort<'a, T>
where
    T: Display,
{
    /// Default formatter: Prints the sorted list joined by commas.
    pub const DEFAULT_FORMATTER: MultiOptionFormatter<'a, T> = &|ans| {
        ans.iter()
            .map(|opt| opt.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    };

    /// Default page size of the list of options.
    pub const DEFAULT_PAGE_SIZE: usize = crate::config::DEFAULT_PAGE_SIZE;

    /// Default setting for Vim mode (disabled by default).
    pub const DEFAULT_VIM_MODE: bool = crate::config::DEFAULT_VIM_MODE;

    /// Default index where the cursor starts.
    pub const DEFAULT_STARTING_CURSOR: usize = 0;

    /// Help message instructing how to reorder the items.
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> =
        Some("[Space] to select, [↑/↓] (or j/k) to reorder, [Enter] to apply");

    /// Creates a [Sort] prompt with the provided message and options.
    pub fn new(message: &'a str, options: Vec<T>) -> Self {
        Self {
            message,
            options,
            help_message: Self::DEFAULT_HELP_MESSAGE,
            page_size: Self::DEFAULT_PAGE_SIZE,
            vim_mode: Self::DEFAULT_VIM_MODE,
            starting_cursor: Self::DEFAULT_STARTING_CURSOR,
            formatter: Self::DEFAULT_FORMATTER,
            render_config: get_configuration(),
        }
    }

    /// Sets the help message to be presented to the user.
    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    /// Removes the help message.
    pub fn without_help_message(mut self) -> Self {
        self.help_message = None;
        self
    }

    /// Sets the page size of the options displayed to the user.
    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size;
        self
    }

    /// Enables or disables Vim-style keybindings.
    pub fn with_vim_mode(mut self, vim_mode: bool) -> Self {
        self.vim_mode = vim_mode;
        self
    }

    /// Sets the starting cursor index.
    pub fn with_starting_cursor(mut self, starting_cursor: usize) -> Self {
        self.starting_cursor = starting_cursor;
        self
    }

    /// Sets the formatter to customize how the final selected answer is printed.
    pub fn with_formatter(mut self, formatter: MultiOptionFormatter<'a, T>) -> Self {
        self.formatter = formatter;
        self
    }

    /// Sets the custom render configuration.
    pub fn with_render_config(mut self, render_config: RenderConfig<'a>) -> Self {
        self.render_config = render_config;
        self
    }

    /// Prompts the user and returns the final sorted list of elements.
    pub fn prompt(self) -> InquireResult<Vec<T>> {
        self.raw_prompt()
            .map(|op| op.into_iter().map(|o| o.value).collect())
    }

    /// Prompts the user and returns the final sorted list of elements,
    /// returning `None` if the operation was canceled.
    pub fn prompt_skippable(self) -> InquireResult<Option<Vec<T>>> {
        match self.prompt() {
            Ok(answer) => Ok(Some(answer)),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Prompts the user and returns the final sorted list of options with original indices,
    /// returning `None` if the operation was canceled.
    pub fn raw_prompt_skippable(self) -> InquireResult<Option<Vec<ListOption<T>>>> {
        match self.raw_prompt() {
            Ok(answer) => Ok(Some(answer)),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Prompts the user and returns the final sorted list of options, preserving original index details.
    pub fn raw_prompt(self) -> InquireResult<Vec<ListOption<T>>> {
        let (input_reader, terminal) = get_default_terminal()?;
        let mut backend = Backend::new(input_reader, terminal, self.render_config)?;
        self.prompt_with_backend(&mut backend)
    }

    /// Prompts the user using the provided rendering backend.
    pub(crate) fn prompt_with_backend<B: SortBackend>(
        self,
        backend: &mut B,
    ) -> InquireResult<Vec<ListOption<T>>> {
        SortPrompt::new(self)?.prompt(backend)
    }
}
