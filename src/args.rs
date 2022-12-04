// SPDX-License-Identifier: GPL-3.0-or-later
//! Command line arguments.

use std::path::PathBuf;

use clap::{ArgGroup, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(group(
    ArgGroup::new("execute")
        .required(false)
        .args(&["exec", "exec_with"]),
))]
pub(crate) struct Args {
    /// Configuration file path.
    pub(crate) config: PathBuf,

    /// Execute the selection.
    #[arg(short = 'x', long)]
    pub(crate) exec: bool,

    /// Execute the selection with the provided command.
    #[arg(short = 'w', long, value_name = "CMD")]
    pub(crate) exec_with: Option<String>,

    /// Exit the program if focus is lost.
    #[arg(short, long)]
    pub(crate) transient: bool,
}
