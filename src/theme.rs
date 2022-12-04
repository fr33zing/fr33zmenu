// SPDX-License-Identifier: GPL-3.0-or-later
//! Theme configuration.
//!
//! See [Theme] to view the accepted fields in a theme configuration.

use std::fmt;

use crossterm::style::Attribute;
use serde::{
    de::{self, Unexpected, Visitor},
    Deserialize, Deserializer,
};

/// Used to deserialize any valid CSS color format into a crossterm color.
#[derive(Debug)]
pub(crate) struct ThemeColor(pub(crate) crossterm::style::Color);

impl Default for ThemeColor {
    fn default() -> Self {
        Self(crossterm::style::Color::Reset)
    }
}

impl<'de> Deserialize<'de> for ThemeColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ThemeColorVisitor;

        impl<'de> Visitor<'de> for ThemeColorVisitor {
            type Value = ThemeColor;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid CSS color")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let css_color = csscolorparser::parse(s)
                    .map_err(|_| de::Error::invalid_value(Unexpected::Str(s), &self))?;
                let crossterm_color = crossterm::style::Color::Rgb {
                    r: (css_color.r * 255.0) as u8,
                    g: (css_color.g * 255.0) as u8,
                    b: (css_color.b * 255.0) as u8,
                };
                Ok(ThemeColor(crossterm_color))
            }
        }
        deserializer.deserialize_str(ThemeColorVisitor)
    }
}

/// Used to deserialize a comma seperated list of text attributes.
#[derive(Debug, Default)]
pub(crate) struct ThemeAttributes(pub(crate) crossterm::style::Attributes);

impl<'de> Deserialize<'de> for ThemeAttributes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ThemeAttributesVisitor;

        impl<'de> Visitor<'de> for ThemeAttributesVisitor {
            type Value = ThemeAttributes;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a comma-separated list of attributes")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let attrs_vec: Vec<Attribute> = s
                    .split(',')
                    .map(|a| match a.trim() {
                        "bold" => Ok(Attribute::Bold),
                        "dim" => Ok(Attribute::Dim),
                        "italic" => Ok(Attribute::Italic),
                        "underlined" => Ok(Attribute::Underlined),
                        "hidden" => Ok(Attribute::Hidden),
                        _ => Err(de::Error::custom(format!(
                            "invalid attribute '{}'",
                            a.trim()
                        ))),
                    })
                    .collect::<Result<_, E>>()?;
                let attrs = crossterm::style::Attributes::from(attrs_vec.as_slice());
                Ok(ThemeAttributes(attrs))
            }
        }
        deserializer.deserialize_str(ThemeAttributesVisitor)
    }
}

/// A text style.
#[derive(Debug, Deserialize, Default)]
pub(crate) struct ThemeStyle {
    /// Foreground color.
    #[serde(default)]
    pub(crate) fg: ThemeColor,

    /// Background color.
    #[serde(default)]
    pub(crate) bg: ThemeColor,

    /// Text attributes.
    #[serde(default)]
    pub(crate) attrs: ThemeAttributes,
}

/// A collection of styles to be used in the interface.
#[derive(Debug, Deserialize)]
pub(crate) struct Theme {
    /// Style for text overflow indicators.
    pub(crate) overflow: ThemeStyle,

    /// Style for the prompt.
    pub(crate) prompt: ThemeStyle,

    /// Style for the user's input.
    pub(crate) input: ThemeStyle,

    /// Style for the name (left side) of a menu entry.
    pub(crate) entry_name: ThemeStyle,

    /// Style for the value (right side) of a menu entry.
    pub(crate) entry_value: ThemeStyle,

    /// Style for letters that match the user's input.
    pub(crate) entry_match: ThemeStyle,

    /// Style for entries that do not match the user's input.
    pub(crate) entry_hidden: ThemeStyle,

    /// Style for the selected entry.
    pub(crate) entry_cursor: ThemeStyle,

    /// Style for letters that match the user's input in the selected entry.
    pub(crate) entry_cursor_match: ThemeStyle,

    /// Style for menu names (i.e. tabs) that are not selected.
    pub(crate) menu_name: ThemeStyle,

    /// Style for the selected menu name.
    pub(crate) menu_cursor: ThemeStyle,
}
