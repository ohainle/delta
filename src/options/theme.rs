//! Delta doesn't have a formal concept of a "theme". What it has is
//!
//! 1. The choice of "theme". This is the language syntax highlighting theme; you have to make this
//!    choice when using `bat` also.
//! 2. The choice of "light vs dark mode". This determines whether the background colors should be
//!    chosen for a light or dark terminal background. (`bat` has no equivalent.)
//!
//! Basically:
//! 1. The theme is specified by the `--syntax-theme` option. If this isn't supplied then it is specified
//!    by the `BAT_THEME` environment variable.
//! 2. Light vs dark mode is specified by the `--light` or `--dark` options. If these aren't
//!    supplied then it detected from the terminal. If this fails it is inferred from the chosen theme.
//!
//! In the absence of other factors, the default assumes a dark terminal background.

use std::io::{stdout, IsTerminal};

use bat;
use bat::assets::HighlightingAssets;
#[cfg(not(test))]
use terminal_colorsaurus::{color_scheme, QueryOptions};

use crate::cli::{self, DetectDarkLight};
use crate::color::{ColorMode, ColorMode::*};

#[allow(non_snake_case)]
pub fn set__color_mode__syntax_theme__syntax_set(opt: &mut cli::Opt, assets: HighlightingAssets) {
    let (color_mode, syntax_theme_name) = get_color_mode_and_syntax_theme_name(
        opt.syntax_theme.as_ref(),
        opt.syntax_theme_light.as_ref(),
        opt.syntax_theme_dark.as_ref(),
        get_color_mode(opt),
    );
    opt.computed.color_mode = color_mode;

    opt.computed.syntax_theme = if is_no_syntax_highlighting_syntax_theme_name(&syntax_theme_name) {
        None
    } else {
        Some(assets.get_theme(&syntax_theme_name).clone())
    };
    opt.computed.syntax_set = assets.get_syntax_set().unwrap().clone();
}

pub fn is_light_syntax_theme(theme: &str) -> bool {
    LIGHT_SYNTAX_THEMES.contains(&theme) || theme.to_lowercase().contains("light")
}

pub fn color_mode_from_syntax_theme(theme: &str) -> ColorMode {
    if is_light_syntax_theme(theme) {
        ColorMode::Light
    } else {
        ColorMode::Dark
    }
}

const LIGHT_SYNTAX_THEMES: [&str; 7] = [
    "Catppuccin Latte",
    "GitHub",
    "gruvbox-light",
    "gruvbox-white",
    "Monokai Extended Light",
    "OneHalfLight",
    "Solarized (light)",
];

const DEFAULT_LIGHT_SYNTAX_THEME: &str = "GitHub";
const DEFAULT_DARK_SYNTAX_THEME: &str = "Monokai Extended";

fn is_no_syntax_highlighting_syntax_theme_name(theme_name: &str) -> bool {
    theme_name.to_lowercase() == "none"
}

/// Return a (theme_name, color_mode) tuple.
/// theme_name == None in return value means syntax highlighting is disabled.
fn get_color_mode_and_syntax_theme_name(
    syntax_theme: Option<&String>,
    syntax_theme_light: Option<&String>,
    syntax_theme_dark: Option<&String>,
    mode: Option<ColorMode>,
) -> (ColorMode, String) {
    match (syntax_theme, mode) {
        (Some(theme), None) => (color_mode_from_syntax_theme(theme), theme.to_string()),
        (Some(theme), Some(mode)) => (mode, theme.to_string()),
        (None, None) => (Dark, DEFAULT_DARK_SYNTAX_THEME.to_string()),
        (None, Some(Dark)) => {
            if let Some(syntax_theme_dark) = syntax_theme_dark {
                (Dark, syntax_theme_dark.to_string())
            } else {
                (Dark, DEFAULT_DARK_SYNTAX_THEME.to_string())
            }
        }
        (None, Some(Light)) => {
            if let Some(syntax_theme_light) = syntax_theme_light {
                (Light, syntax_theme_light.to_string())
            } else {
                (Light, DEFAULT_LIGHT_SYNTAX_THEME.to_string())
            }
        }
    }
}

fn get_color_mode(opt: &cli::Opt) -> Option<ColorMode> {
    if opt.light {
        Some(Light)
    } else if opt.dark {
        Some(Dark)
    } else if should_detect_color_mode(opt) {
        detect_color_mode()
    } else {
        None
    }
}

/// See [`cli::Opt::detect_dark_light`] for a detailed explanation.
fn should_detect_color_mode(opt: &cli::Opt) -> bool {
    match opt.detect_dark_light {
        DetectDarkLight::Auto => opt.color_only || stdout().is_terminal(),
        DetectDarkLight::Always => true,
        DetectDarkLight::Never => false,
    }
}

#[cfg(not(test))]
fn detect_color_mode() -> Option<ColorMode> {
    color_scheme(QueryOptions::default())
        .ok()
        .map(ColorMode::from)
}

impl From<terminal_colorsaurus::ColorScheme> for ColorMode {
    fn from(value: terminal_colorsaurus::ColorScheme) -> Self {
        match value {
            terminal_colorsaurus::ColorScheme::Dark => ColorMode::Dark,
            terminal_colorsaurus::ColorScheme::Light => ColorMode::Light,
        }
    }
}

#[cfg(test)]
fn detect_color_mode() -> Option<ColorMode> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color;
    use crate::tests::integration_test_utils;

    struct SyntaxThemeTestCase {
        syntax_theme: Option<&'static str>,
        syntax_theme_dark: Option<&'static str>,
        syntax_theme_light: Option<&'static str>,
        mode: Option<ColorMode>,
        expected_syntax_theme: &'static str,
        expected_mode: ColorMode,
    }

    // TODO: Test influence of BAT_THEME env var. E.g. see utils::process::tests::FakeParentArgs.
    #[test]
    fn test_syntax_theme_selection() {
        let _cases = vec![
            // when no theme or mode specifified,
            // select the default dark theme and dark mode
            SyntaxThemeTestCase {
                syntax_theme: None,
                syntax_theme_dark: None,
                syntax_theme_light: None,
                mode: None,
                expected_syntax_theme: DEFAULT_DARK_SYNTAX_THEME,
                expected_mode: Dark,
            },
            // when the specified theme is light and no mode is specified,
            // select the specified theme and light mode
            SyntaxThemeTestCase {
                syntax_theme: Some("GitHub"),
                syntax_theme_dark: None,
                syntax_theme_light: None,
                mode: None,
                expected_syntax_theme: "GitHub",
                expected_mode: Light,
            },
            // when the specified theme is dark and no mode is specified,
            // select the specified theme and dark mode
            SyntaxThemeTestCase {
                syntax_theme: Some("Nord"),
                syntax_theme_dark: None,
                syntax_theme_light: None,
                mode: None,
                expected_syntax_theme: "Nord",
                expected_mode: Dark,
            },
            // when no theme is specified and mode is specified as light,
            // select the default light theme and light mode
            SyntaxThemeTestCase {
                syntax_theme: None,
                syntax_theme_dark: None,
                syntax_theme_light: None,
                mode: Some(Light),
                expected_syntax_theme: DEFAULT_LIGHT_SYNTAX_THEME,
                expected_mode: Light,
            },
            // when no theme is specified and mode is specified as dark,
            // select the default dark theme and dark mode
            SyntaxThemeTestCase {
                syntax_theme: None,
                syntax_theme_dark: None,
                syntax_theme_light: None,
                mode: Some(Dark),
                expected_syntax_theme: DEFAULT_DARK_SYNTAX_THEME,
                expected_mode: Dark,
            },
            // when theme is specified and mode is specified
            // select the specified theme and specified mode
            SyntaxThemeTestCase {
                syntax_theme: Some("GitHub"),
                syntax_theme_dark: None,
                syntax_theme_light: None,
                mode: Some(Light),
                expected_syntax_theme: "GitHub",
                expected_mode: Light,
            },
            SyntaxThemeTestCase {
                syntax_theme: Some("GitHub"),
                syntax_theme_dark: None,
                syntax_theme_light: None,
                mode: Some(Light),
                expected_syntax_theme: "GitHub",
                expected_mode: Light,
            },
            SyntaxThemeTestCase {
                syntax_theme: Some("Nord"),
                syntax_theme_dark: None,
                syntax_theme_light: None,
                mode: Some(Light),
                expected_syntax_theme: "Nord",
                expected_mode: Light,
            },
            SyntaxThemeTestCase {
                syntax_theme: Some("Nord"),
                syntax_theme_dark: None,
                syntax_theme_light: None,
                mode: Some(Light),
                expected_syntax_theme: "Nord",
                expected_mode: Light,
            },
            // when theme is specified as 'none' and mode is not specified
            // select the 'none' theme and default to dark mode
            SyntaxThemeTestCase {
                syntax_theme: Some("none"),
                syntax_theme_dark: None,
                syntax_theme_light: None,
                mode: None,
                expected_syntax_theme: "none",
                expected_mode: Dark,
            },
            SyntaxThemeTestCase {
                syntax_theme: Some("none"),
                syntax_theme_dark: None,
                syntax_theme_light: None,
                mode: Some(Dark),
                expected_syntax_theme: "none",
                expected_mode: Dark,
            },
            SyntaxThemeTestCase {
                syntax_theme: Some("None"),
                syntax_theme_dark: None,
                syntax_theme_light: None,
                mode: Some(Light),
                expected_syntax_theme: "none",
                expected_mode: Light,
            },
        ];

        for SyntaxThemeTestCase {
            syntax_theme,
            syntax_theme_dark,
            syntax_theme_light,
            mode,
            expected_syntax_theme,
            expected_mode,
        } in _cases
        {
            let mut args = vec![];
            if let Some(syntax_theme) = syntax_theme {
                args.push("--syntax-theme");
                args.push(syntax_theme);
            }
            if let Some(syntax_theme_dark) = syntax_theme_dark {
                args.push("--syntax-theme-dark");
                args.push(syntax_theme_dark);
            }
            if let Some(syntax_theme_light) = syntax_theme_light {
                args.push("--syntax-theme-light");
                args.push(syntax_theme_light);
            }
            let is_true_color = true;
            if is_true_color {
                args.push("--true-color");
                args.push("always");
            } else {
                args.push("--true-color");
                args.push("never");
            }
            match mode {
                Some(Light) => {
                    args.push("--light");
                }
                Some(Dark) => {
                    args.push("--dark");
                }
                None => {}
            }
            let config = integration_test_utils::make_config_from_args(&args);
            assert_eq!(
                &config
                    .syntax_theme
                    .clone()
                    .map(|t| t.name.unwrap())
                    .unwrap_or("none".to_string()),
                expected_syntax_theme
            );
            if is_no_syntax_highlighting_syntax_theme_name(expected_syntax_theme) {
                assert!(config.syntax_theme.is_none())
            } else {
                assert_eq!(
                    config.syntax_theme.unwrap().name.as_ref().unwrap(),
                    expected_syntax_theme
                );
            }
            assert_eq!(
                config.minus_style.ansi_term_style.background.unwrap(),
                color::get_minus_background_color_default(expected_mode, is_true_color)
            );
            assert_eq!(
                config.minus_emph_style.ansi_term_style.background.unwrap(),
                color::get_minus_emph_background_color_default(expected_mode, is_true_color)
            );
            assert_eq!(
                config.plus_style.ansi_term_style.background.unwrap(),
                color::get_plus_background_color_default(expected_mode, is_true_color)
            );
            assert_eq!(
                config.plus_emph_style.ansi_term_style.background.unwrap(),
                color::get_plus_emph_background_color_default(expected_mode, is_true_color)
            );
        }
    }
}
