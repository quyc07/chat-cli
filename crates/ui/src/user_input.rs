//! # [Ratatui] User Input example
//!
//! The latest version of this example is available in the [examples] folder in the repository.
//!
//! Please note that the examples are designed to be run against the `main` branch of the Github
//! repository. This means that you may not be able to compile with the latest release version on
//! crates.io, or the one that you have installed locally.
//!
//! See the [examples readme] for more information on finding examples that match the version of the
//! library you are using.
//!
//! [Ratatui]: https://github.com/ratatui/ratatui
//! [examples]: https://github.com/ratatui/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui/ratatui/blob/main/examples/README.md

// A simple example demonstrating how to handle user input. This is a bit out of the scope of
// the library as it does not provide any input handling out of the box. However, it may helps
// some to get started.
//
// This is a very simple example:
//   * An input box always focused. Every character you type is registered here.
//   * An entered character is inserted at the cursor position.
//   * Pressing Backspace erases the left character before the cursor position
//   * Pressing Enter pushes the current input in the history of previous messages. **Note: ** as
//   this is a relatively simple example unicode characters are unsupported and their use will
// result in undefined behaviour.
//
// See also https://github.com/rhysd/tui-textarea and https://github.com/sayanarijit/tui-input/

/// App holds the state of the application
pub struct Input {
    /// Current value of the input box
    pub(crate) input: String,
    /// Position of cursor in the editor area.
    pub(crate) character_index: usize,
    /// Current input mode
    pub(crate) current_mode: CurrentMode,
    /// History of recorded messages
    pub(crate) messages: Vec<String>,
}

pub(crate) enum CurrentMode {
    Normal,
    Editing,
    Alerting,
}

impl Input {
    pub(crate) const fn new() -> Self {
        Self {
            input: String::new(),
            current_mode: CurrentMode::Normal,
            messages: Vec::new(),
            character_index: 0,
        }
    }

    pub(crate) fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub(crate) fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub(crate) fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    pub(crate) fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub(crate) fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub(crate) fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    pub(crate) fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    pub(crate) fn submit_message(&mut self) {
        self.messages.push(self.input.clone());
        self.input.clear();
        self.reset_cursor();
    }

}