// SPDX-License-Identifier: GPL-3.0-or-later
//! Loads and globs configuration files.

use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result};

use serde::Deserialize;
use serde_with::serde_as;

use crate::{keybinds::Keybinds, theme::Theme};

static DEFAULT_THEME: &'static str = include_str!("../config/theme.default.toml");
static DEFAULT_KEYBINDS: &'static str = include_str!("../config/keybinds.default.toml");

/// A menu page.
#[serde_as]
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct Menu {
    /// The sorting order.
    #[serde(default)]
    pub(crate) order: i64,

    /// The input prompt.
    pub(crate) prompt: String,

    /// The menu's entries. The key is used as the entry name.
    #[serde_as(as = "HashMap<_, _>")]
    pub(crate) entries: Vec<(String, String)>,
}

/// A configuration file.
#[serde_as]
#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    /// A collection of styles to be used in the interface.
    pub(crate) theme: Theme,

    /// Pages of entries. The key is used as the menu name.
    #[serde_as(as = "HashMap<_, _>")]
    pub(crate) menus: Vec<(String, Menu)>,

    /// Keybinds used to interact with the interface.
    pub(crate) keybinds: Keybinds,
}

/// Loads the provided config file, and combines it with the defaults.
pub(crate) fn load_config(file: PathBuf) -> Result<Config> {
    let builder = config::Config::builder()
        .add_source(config::File::from_str(
            DEFAULT_THEME,
            config::FileFormat::Toml,
        ))
        .add_source(config::File::from_str(
            DEFAULT_KEYBINDS,
            config::FileFormat::Toml,
        ))
        .add_source(config::File::from(file.clone()));
    let config = builder
        .build()
        .context("Failed to read config sources")?
        .try_deserialize::<Config>()
        .context("Failed to deserialize config")?;
    Ok(config)
}
