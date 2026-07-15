use crate::prompts::sort::Sort;

/// Configuration settings used in the execution of a SortPrompt.
#[derive(Copy, Clone, Debug)]
pub struct SortConfig {
    /// Whether to use vim-style keybindings.
    pub vim_mode: bool,
    /// Page size of the list of options.
    pub page_size: usize,
}

impl<T> From<&Sort<'_, T>> for SortConfig {
    fn from(value: &Sort<'_, T>) -> Self {
        Self {
            vim_mode: value.vim_mode,
            page_size: value.page_size,
        }
    }
}
