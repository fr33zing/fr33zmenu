// SPDX-License-Identifier: GPL-3.0-or-later
//! Utility functions.

use std::{fs, io};

use crossterm::terminal;
use fuzzy_matcher::clangd::fuzzy_indices;

use crate::{config::Config, state::State};

pub(crate) fn tty() -> io::Result<fs::File> {
    fs::OpenOptions::new()
        .read(false)
        .write(true)
        .open("/dev/tty")
}

pub(crate) fn sort_menus(config: &mut Config) {
    config.menus.sort_by(|a, b| {
        if a.1.order == b.1.order {
            a.0.to_lowercase().cmp(&b.0.to_lowercase())
        } else {
            a.1.order.cmp(&b.1.order)
        }
    });
}

pub(crate) fn match_entries(
    input: &str,
    entries: &Vec<(String, String)>,
) -> Vec<(Option<(i64, Vec<usize>)>, String, String)> {
    let mut entries_sorted: Vec<(Option<(i64, Vec<usize>)>, String, String)> = entries
        .iter()
        .map(|entry| {
            (
                fuzzy_indices(&entry.0, input),
                entry.0.clone(),
                entry.1.clone(),
            )
        })
        .collect();

    if input.is_empty() {
        entries_sorted.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
    } else {
        entries_sorted.sort_by(|a, b| {
            if a.0.is_none() && b.0.is_none() {
                a.1.to_lowercase().cmp(&b.1.to_lowercase())
            } else {
                b.0.cmp(&a.0)
            }
        });
    }

    entries_sorted
}

pub(crate) fn count_selectable_entries(
    state: &State,
    entries: &Vec<(Option<(i64, Vec<usize>)>, String, String)>,
) -> usize {
    let h: usize = match terminal::size() {
        Ok(size) => size.1.into(),
        Err(_) => return 0,
    };

    let count = if state.input.is_empty() {
        entries.len()
    } else {
        entries
            .iter()
            .filter_map(|e| match &e.0 {
                Some(e) => Some(e),
                None => None,
            })
            .count()
    };

    usize::min(count, h.saturating_sub(5))
}
