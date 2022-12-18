// SPDX-License-Identifier: GPL-3.0-or-later
//! A multi-page fuzzy launcher for your terminal.

use std::{
    io::{self, stderr, stdout, Write},
    process::{self, Command, Stdio},
    time::Duration,
};

use anyhow::{anyhow, Result};
use args::Args;
use clap::Parser;
use crossterm::{
    cursor::{MoveTo, SavePosition},
    event::{poll, read, DisableFocusChange, EnableFocusChange, Event},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};

mod args;
mod config;
mod draw;
mod keybinds;
mod macros;
mod state;
mod theme;
mod util;

use crate::{
    draw::draw,
    state::{Action, State},
};

fn main() {
    let res: Result<()> = (|| {
        let mut tty = util::tty()?;
        let args = args::Args::parse();
        let mut config = config::load_config(args.config.clone())?;
        util::sort_menus(&mut config);
        execute!(tty, Clear(ClearType::All), EnableFocusChange)?;
        enable_raw_mode()?;
        let selection = interact(&mut tty, &args, &mut config)?;
        disable_raw_mode()?;
        submit(&mut tty, &args, selection)?;
        execute!(tty, Clear(ClearType::All), MoveTo(0, 0), DisableFocusChange)?;
        Ok(())
    })();

    match res {
        Ok(_) => process::exit(0),
        Err(e) => {
            let _ = writeln!(stderr(), "{e:?}");
            process::exit(1);
        }
    }
}

/// Handles event polling, state management, and drawing the interface.
fn interact(tty: &mut impl io::Write, args: &Args, config: &mut config::Config) -> Result<String> {
    let mut first = true;
    let mut state = State::default();
    state.menu_count = config.menus.len().try_into()?;

    loop {
        let last_state = state.clone();
        let mut force_redraw = false;

        // Handle events
        if !first {
            if !poll(Duration::from_millis(100))? {
                continue;
            }

            match read()? {
                Event::Resize(_, _) => {
                    force_redraw = true;
                }
                Event::FocusLost => {
                    if args.transient {
                        break;
                    }
                }
                Event::Key(event) => {
                    execute!(tty, SavePosition)?;
                    state = config.keybinds.handle(event, state)?;
                }
                _ => {}
            }
        }

        // Update + draw
        if state != last_state || first || force_redraw {
            // Update
            let menu = config
                .menus
                .get(state.menu_index)
                .ok_or_else(|| anyhow!("invalid menu index"))?;
            let entries = util::match_entries(&state.input, &menu.1.entries);
            state.entry_count = util::count_selectable_entries(&state, &entries);

            // Handle state action
            match state.action {
                Action::None => {}
                Action::Exit => break,
                Action::Clear => {
                    execute!(tty, Clear(ClearType::All))?;
                }
                Action::Submit => {
                    if state.entry_count > 0 {
                        let selection = entries
                            .get(state.entry_index)
                            .ok_or_else(|| anyhow!("selection index out of bounds"))?;
                        return Ok(selection.2.clone());
                    }
                }
            }

            state.action = Action::Clear;
            first = false;
            draw(tty, config, &mut state, menu, &entries)?;
            tty.flush()?;
        }
    }
    Ok(String::default())
}

/// Writes the selected entry's value to stdout, or if `--exec` / `--exec-with` is provided,
/// executes it.
// TODO clean this up
fn submit(tty: &mut impl io::Write, args: &Args, selection: String) -> Result<()> {
    execute!(tty, Clear(ClearType::All), MoveTo(0, 0))?;
    if args.exec {
        // --exec
        Command::new("nohup")
            .arg(selection)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
    } else if let Some(e) = &args.exec_with {
        // --exec-with
        let mut split = e.split(" ");
        let cmd = split.next().ok_or_else(|| anyhow!("empty exec_with"))?;
        let executor_args: Vec<&str> = split.collect();
        Command::new(cmd)
            .args(executor_args)
            .arg(selection)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
    } else {
        execute!(stdout(), Print(&selection), Print('\n'))?;
        return Ok(());
    }

    Ok(())
}
