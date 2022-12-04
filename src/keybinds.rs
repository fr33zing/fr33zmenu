// SPDX-License-Identifier: GPL-3.0-or-later
//! Keybind configuration and key event handling.

use std::fmt;

use anyhow::{anyhow, bail, Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

use crate::{
    handle_key_event,
    state::{Action, State},
};

/// Indicates that unhandled key events should cause errors.
const UNHANDLED_KEY_EVENT_ERRORS: bool = false;

#[derive(Debug)]
/// Used to deserialize keybinds from a plus-seperated list of modifier keys and one non-modifier
/// key.
pub(crate) struct Keybind(
    /// **One** non-modifier key.
    pub(crate) KeyCode,
    /// Zero, one, or multiple modifier keys.
    pub(crate) KeyModifiers,
);

impl Keybind {
    fn matches(&self, event: KeyEvent) -> bool {
        let code = if event.code == KeyCode::BackTab {
            KeyCode::Tab
        } else {
            event.code
        };
        code == self.0 && event.modifiers == self.1
    }
}

impl<'de> Deserialize<'de> for Keybind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeybindVisitor;

        impl<'de> Visitor<'de> for KeybindVisitor {
            type Value = Keybind;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a plus-separated list of attributes")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let mut code: Option<KeyCode> = None;
                let mut modifiers = KeyModifiers::from_bits(0).unwrap();

                for key in s.split('+') {
                    let key = key.trim().to_lowercase();
                    let key = key.as_str();
                    let mut c: Option<KeyCode> = None;

                    match key {
                        "shift" => modifiers.insert(KeyModifiers::SHIFT),
                        "control" | "ctrl" => modifiers.insert(KeyModifiers::CONTROL),
                        "alt" => modifiers.insert(KeyModifiers::ALT),
                        "backspace" | "back" => c = Some(KeyCode::Backspace),
                        "enter" | "return" | "ret" => c = Some(KeyCode::Enter),
                        "left" => c = Some(KeyCode::Left),
                        "right" => c = Some(KeyCode::Right),
                        "up" => c = Some(KeyCode::Up),
                        "down" => c = Some(KeyCode::Down),
                        "home" => c = Some(KeyCode::Home),
                        "end" => c = Some(KeyCode::End),
                        "pageup" | "pgup" => c = Some(KeyCode::PageUp),
                        "pagedown" | "pgdn" => c = Some(KeyCode::PageDown),
                        "tab" => c = Some(KeyCode::Tab),
                        "delete" | "del" => c = Some(KeyCode::Delete),
                        "insert" => c = Some(KeyCode::Insert),
                        "escape" | "esc" => c = Some(KeyCode::Esc),
                        _ => {
                            let mut chars = key.chars();
                            if let Some(first_char) = chars.next() {
                                if key.len() == 1 {
                                    c = Some(KeyCode::Char(first_char));
                                } else if first_char == 'f' {
                                    let remaining: String = chars.collect();
                                    let num = remaining.parse::<u8>().map_err(|_| {
                                        de::Error::custom("invalid function key code")
                                    })?;
                                    c = Some(KeyCode::F(num));
                                }
                            } else {
                                return Err(de::Error::custom("empty key code"));
                            }
                        }
                    };

                    if let Some(c) = c {
                        if code.is_some() {
                            return Err(de::Error::custom("multiple non-modifier keys"));
                        } else {
                            code = Some(c);
                        }
                    }
                }

                if let Some(code) = code {
                    Ok(Keybind(code, modifiers))
                } else {
                    Err(de::Error::custom(
                        "keybind must include one non-modifier key",
                    ))
                }
            }
        }
        deserializer.deserialize_str(KeybindVisitor)
    }
}

/// A collection of keybinds used to control the program.
#[derive(Debug, Deserialize)]
pub(crate) struct Keybinds {
    /// Quit the program.
    pub(crate) exit: Vec<Keybind>,

    /// Submit / execute the selected entry.
    pub(crate) submit: Vec<Keybind>,

    /// Clear the input.
    pub(crate) clear: Vec<Keybind>,

    /// Delete the next character at the input cursor.
    pub(crate) delete_next: Vec<Keybind>,

    /// Delete the previous character at the input cursor.
    pub(crate) delete_back: Vec<Keybind>,

    /// Move the input cursor to the right.
    pub(crate) input_next: Vec<Keybind>,

    /// Move the input cursor to the left.
    pub(crate) input_back: Vec<Keybind>,

    /// Go to the next menu to the right.
    pub(crate) menu_next: Vec<Keybind>,

    /// Go to the previous menu to the left.
    pub(crate) menu_back: Vec<Keybind>,

    /// Select the next entry.
    pub(crate) entry_next: Vec<Keybind>,

    /// Select the previous entry.
    pub(crate) entry_back: Vec<Keybind>,
}

impl Keybinds {
    pub(crate) fn handle(&self, event: KeyEvent, state: State) -> Result<State> {
        let (handled, state_res) = handle_key_event!(
            self,
            event,
            state,
            [
                exit,
                submit,
                clear,
                delete_next,
                delete_back,
                input_next,
                input_back,
                entry_next,
                entry_back,
                menu_next,
                menu_back
            ]
        );
        let state = state_res.context("Keybind handler error")?;
        if handled {
            Ok(state)
        } else {
            Keybinds::fallback_handler(state, event)
        }
    }

    fn fallback_handler(state: State, event: KeyEvent) -> Result<State> {
        let new_state = match event.code {
            KeyCode::Char(c) => {
                if event.modifiers.bits() <= 1 {
                    let mut input = state.input.clone();
                    input.insert(state.cursor_x.into(), c);
                    let state = State {
                        input,
                        cursor_x: state.cursor_x.saturating_add(1),
                        entry_cursor: false,
                        entry_index: 0,
                        ..state.clone()
                    };
                    Some(state)
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(state) = new_state {
            Ok(state)
        } else if UNHANDLED_KEY_EVENT_ERRORS {
            bail!("Unhandled key event: {event:?}");
        } else {
            Ok(state)
        }
    }

    fn exit(state: State) -> Result<State> {
        let state = State {
            action: Action::Exit,
            ..state
        };
        Ok(state)
    }

    fn submit(state: State) -> Result<State> {
        let state = State {
            action: Action::Submit,
            ..state
        };
        Ok(state)
    }

    fn clear(state: State) -> Result<State> {
        let state = State {
            input: String::default(),
            cursor_x: 0,
            entry_cursor: false,
            ..state
        };
        Ok(state)
    }

    fn delete_next(state: State) -> Result<State> {
        let state = State {
            entry_cursor: false,
            input: state
                .input
                .char_indices()
                .filter_map(|(i, c)| {
                    if (i as u16) == state.cursor_x {
                        None
                    } else {
                        Some(c)
                    }
                })
                .collect(),
            ..state
        };
        Ok(state)
    }

    fn delete_back(state: State) -> Result<State> {
        if state.cursor_x == 0 {
            return Ok(state);
        }
        let cursor_x = state.cursor_x.saturating_sub(1);
        let state = State {
            entry_cursor: false,
            input: state
                .input
                .char_indices()
                .filter_map(|(i, c)| {
                    if (i as u16) == cursor_x {
                        None
                    } else {
                        Some(c)
                    }
                })
                .collect(),
            cursor_x,
            ..state
        };
        Ok(state)
    }

    fn input_next(state: State) -> Result<State> {
        let len: u16 = state.input.len().try_into()?;
        let state = State {
            cursor_x: u16::min(len, state.cursor_x.saturating_add(1)),
            ..state
        };
        Ok(state)
    }

    fn input_back(state: State) -> Result<State> {
        let state = State {
            cursor_x: u16::max(0, state.cursor_x.saturating_sub(1)),
            ..state
        };
        Ok(state)
    }

    fn entry_next(state: State) -> Result<State> {
        if state.entry_count == 0 {
            return Ok(state);
        }
        let state = State {
            entry_cursor: true,
            entry_index: if state.entry_cursor {
                state.entry_index.saturating_add(1) % state.entry_count
            } else {
                0
            },
            ..state
        };
        Ok(state)
    }

    fn entry_back(state: State) -> Result<State> {
        if state.entry_count == 0 {
            return Ok(state);
        }
        let state = State {
            entry_cursor: true,
            entry_index: if state.entry_cursor && state.entry_index != 0 {
                state.entry_index.saturating_sub(1) % state.entry_count
            } else {
                state.entry_count - 1
            },
            ..state
        };
        Ok(state)
    }

    fn menu_next(state: State) -> Result<State> {
        let state = State {
            input: String::default(),
            cursor_x: 0,
            entry_cursor: false,
            entry_index: 0,
            menu_index: state
                .menu_index
                .saturating_add(1)
                .checked_rem(state.menu_count)
                .ok_or_else(|| anyhow!("zero menus"))?,
            ..state
        };
        Ok(state)
    }

    fn menu_back(state: State) -> Result<State> {
        let state = State {
            input: String::default(),
            cursor_x: 0,
            entry_cursor: false,
            entry_index: 0,
            menu_index: state
                .menu_index
                .saturating_sub(1)
                .checked_rem(state.menu_count)
                .ok_or_else(|| anyhow!("zero menus"))?,
            ..state
        };
        Ok(state)
    }
}
