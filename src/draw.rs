// SPDX-License-Identifier: GPL-3.0-or-later
//! Draws the interface.

use anyhow::Context;
use crossterm::{
    cursor::{MoveRight, MoveTo, MoveToColumn, MoveToNextLine, RestorePosition, SavePosition},
    execute, queue,
    style::{Print, ResetColor, SetAttributes, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};

use crate::{
    config::{Config, Menu},
    set_style,
    state::State,
    theme::Theme,
};

// Spacing between elements on the same line
const SPACING: u16 = 2;

const ROW_MENULINE: u16 = 0;
const ROW_PROMPT: u16 = 2;
const ROW_ENTRIES: u16 = 4;

/// Draws the interface.
pub(crate) fn draw(
    tty: &mut impl std::io::Write,
    config: &Config,
    state: &mut State,
    menu: &(String, Menu),
    entries: &Vec<(Option<(i64, Vec<usize>)>, String, String)>,
) -> Result<(), anyhow::Error> {
    draw_menu_line(tty, &config.theme, &config.menus, state.menu_index)
        .context("Failed to draw menu line")?;
    draw_entries(
        tty,
        &config.theme,
        &entries,
        state.entry_cursor,
        state.entry_index,
    )
    .context("Failed to draw entries")?;
    draw_prompt(tty, &config.theme, &menu.1.prompt).context("Failed to draw prompt")?;
    draw_input(tty, &config.theme, &state.input, state.cursor_x)
        .context("Failed to draw user input")?;

    Ok(())
}

fn draw_menu_line(
    tty: &mut impl std::io::Write,
    theme: &Theme,
    menus: &Vec<(String, Menu)>,
    menu_index: usize,
) -> anyhow::Result<()> {
    let mut x: u16 = 0;
    for (i, menu) in menus.iter().enumerate() {
        let style = if i == menu_index {
            &theme.menu_cursor
        } else {
            &theme.menu_name
        };

        execute!(
            tty,
            ResetColor,
            MoveTo(x, ROW_MENULINE),
            set_style!(style),
            Print(&menu.0)
        )?;
        x += &menu.0.len().try_into()? + SPACING;
    }
    Ok(())
}

fn draw_prompt(
    tty: &mut impl std::io::Write,
    theme: &Theme,
    text: &str,
) -> Result<(), std::io::Error> {
    execute!(
        tty,
        MoveTo(0, ROW_PROMPT),
        ResetColor,
        set_style!(theme.prompt),
        Print(text),
        ResetColor,
        SavePosition
    )
}

fn draw_input(
    tty: &mut impl std::io::Write,
    theme: &Theme,
    text: &str,
    cursor_x: u16,
) -> Result<(), anyhow::Error> {
    execute!(
        tty,
        RestorePosition,
        Clear(ClearType::UntilNewLine),
        set_style!(theme.input),
        Print(text),
        RestorePosition
    )?;
    if cursor_x > 0 {
        execute!(tty, MoveRight(cursor_x))?;
    }
    Ok(())
}

fn draw_entries(
    tty: &mut impl std::io::Write,
    theme: &Theme,
    entries: &Vec<(Option<(i64, Vec<usize>)>, String, String)>,
    entry_cursor: bool,
    entry_index: usize,
) -> anyhow::Result<()> {
    queue!(tty, MoveTo(0, ROW_ENTRIES), ResetColor)?;

    let size = terminal::size()?;
    let w = size.0;
    let h: usize = size.1.try_into()?;

    for (i, entry) in entries.iter().enumerate() {
        let y = i + 2;

        if y < h {
            let selected = entry_cursor && i == entry_index;
            draw_entry(tty, theme, w, entry, selected)?;
        } else if i == 0 {
            break; // No room to draw anything
        } else {
            let msg = format!("+{} more", entries.len() - i + 1);
            queue!(
                tty,
                set_style!(theme.overflow),
                MoveToNextLine(1),
                MoveToNextLine(1),
                Clear(ClearType::CurrentLine),
                Print(msg)
            )?;
            break;
        }
    }

    Ok(())
}

fn draw_entry(
    tty: &mut impl std::io::Write,
    theme: &Theme,

    term_width: u16,
    entry: &(Option<(i64, Vec<usize>)>, String, String),
    selected: bool,
) -> Result<(), anyhow::Error> {
    if let Some(fuzzy) = &entry.0 {
        for (j, c) in entry.1.char_indices() {
            let style = if fuzzy.1.contains(&j) {
                if selected {
                    &theme.entry_cursor_match
                } else {
                    &theme.entry_match
                }
            } else {
                if selected {
                    &theme.entry_cursor
                } else {
                    &theme.entry_name
                }
            };
            queue!(tty, ResetColor, set_style!(style), Print(c))?;
        }
    } else {
        queue!(
            tty,
            ResetColor,
            SetForegroundColor(theme.entry_hidden.fg.0),
            SetAttributes(theme.entry_hidden.attrs.0),
            Print(&entry.1)
        )?;
    }

    // Draw value on right side
    let name_width: u16 = entry.1.len().try_into()?;
    let name_width = name_width + SPACING;
    let value_width: u16 = entry.2.len().try_into()?;
    let remaining_cols = term_width.saturating_sub(name_width);

    let style = match entry.0 {
        Some(_) => &theme.entry_value,
        None => &theme.entry_hidden,
    };

    if remaining_cols >= value_width {
        queue!(
            tty,
            ResetColor,
            set_style!(style),
            MoveToColumn(term_width - value_width),
            Print(&entry.2)
        )?;
    } else if remaining_cols >= 4 {
        // at least 1 char + ellipses
        let overflow_indicator = "+";
        let overflow_indicator_width: u16 = overflow_indicator.len().try_into()?;
        let value_trunc = entry
            .2
            .get(..(remaining_cols - overflow_indicator_width).into());

        if let Some(vt) = value_trunc {
            let value_trunc_width: u16 = vt.len().try_into()?;
            let value_total_width: u16 = value_trunc_width + overflow_indicator_width;
            queue!(
                tty,
                ResetColor,
                set_style!(style),
                MoveToColumn(term_width - value_total_width),
                Print(vt),
                set_style!(theme.overflow),
                Print(overflow_indicator)
            )?;
        }
    }
    queue!(tty, MoveToNextLine(1), MoveToColumn(0))?;
    Ok(())
}
