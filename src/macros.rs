// SPDX-License-Identifier: GPL-3.0-or-later

#[macro_export]
macro_rules! set_style {
    ($style:expr) => {
        crossterm::style::SetStyle(crossterm::style::ContentStyle {
            foreground_color: Some($style.fg.0),
            background_color: Some($style.bg.0),
            underline_color: None,
            attributes: $style.attrs.0,
        })
    };
}

#[macro_export]
macro_rules! handle_key_event {
    ( $self:ident, $event:ident, $state:ident, [$( $bind:ident ),+] ) => {
        'x: {
            $(
                if $self.$bind.iter().any(|kb| kb.matches($event)) {
                    break 'x (true, Keybinds::$bind($state));
                }
            )*
            (false, Ok($state))
        }
    };
}
