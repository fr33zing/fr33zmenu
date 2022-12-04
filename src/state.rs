// SPDX-License-Identifier: GPL-3.0-or-later

/// Indicates the next action the program should take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Action {
    /// Indicates that the program should continue.
    None,

    /// Indicates that the program should exit without submitting.
    Exit,

    /// Indicates that the screen should be cleared.
    Clear,

    /// Indicates that the program should submit the selected entry and exit.
    Submit,
}

impl Default for Action {
    fn default() -> Self {
        Action::None
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct State {
    /// The user's query.
    pub(crate) input: String,

    /// Indicates the next action the program should take.
    pub(crate) action: Action,

    /// Position of the input cursor, offset from the left.
    pub(crate) cursor_x: u16,

    /// Indicates that the entry cursor is visible.
    pub(crate) entry_cursor: bool,

    /// The number of selectable entries in the current menu.
    pub(crate) entry_count: usize,

    /// Index of the selected entry.
    pub(crate) entry_index: usize,

    /// The number of menus in the config.
    pub(crate) menu_count: usize,

    /// Index of the current menu.
    pub(crate) menu_index: usize,
}
