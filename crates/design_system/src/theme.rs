use gpui::{App, AppContext, Context, Entity, Global, Rgba, rgb};

use crate::tokens::color;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ThemeMode {
    Light,
    #[default]
    Dark,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemeColors {
    pub background: Rgba,
    pub surface: Rgba,
    pub surface_elevated: Rgba,
    pub text_primary: Rgba,
    pub text_muted: Rgba,
    pub border: Rgba,
    pub accent: Rgba,
    pub accent_hover: Rgba,
    pub on_accent: Rgba,
    pub danger: Rgba,
    pub danger_hover: Rgba,
    pub focus_ring: Rgba,
    pub shadow: Rgba,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub mode: ThemeMode,
    pub colors: ThemeColors,
}

impl Theme {
    pub fn new(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self::light(),
            ThemeMode::Dark => Self::dark(),
        }
    }

    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            colors: ThemeColors {
                background: rgb(color::NEUTRAL_50),
                surface: rgb(color::WHITE),
                surface_elevated: rgb(color::NEUTRAL_100),
                text_primary: rgb(color::NEUTRAL_900),
                text_muted: rgb(color::NEUTRAL_600),
                border: rgb(color::NEUTRAL_200),
                accent: rgb(color::BLUE_600),
                accent_hover: rgb(color::BLUE_700),
                on_accent: rgb(color::WHITE),
                danger: rgb(color::RED_600),
                danger_hover: rgb(color::RED_700),
                focus_ring: rgb(color::BLUE_600),
                shadow: rgb(color::NEUTRAL_950),
            },
        }
    }

    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            colors: ThemeColors {
                background: rgb(color::DARK_BACKGROUND),
                surface: rgb(color::NEUTRAL_900),
                surface_elevated: rgb(color::NEUTRAL_800),
                text_primary: rgb(color::NEUTRAL_50),
                text_muted: rgb(color::NEUTRAL_400),
                border: rgb(color::NEUTRAL_700),
                accent: rgb(color::BLUE_500),
                accent_hover: rgb(color::BLUE_400),
                on_accent: rgb(color::WHITE),
                danger: rgb(color::RED_500),
                danger_hover: rgb(color::RED_400),
                focus_ring: rgb(color::BLUE_500),
                shadow: rgb(color::NEUTRAL_950),
            },
        }
    }

    pub fn set_mode(&mut self, mode: ThemeMode, cx: &mut Context<Self>) {
        *self = Self::new(mode);
        cx.notify();
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        let next = match self.mode {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        };
        self.set_mode(next, cx);
    }
}

#[derive(Clone, Debug)]
pub struct ActiveTheme(pub Entity<Theme>);

impl ActiveTheme {
    pub fn init(mode: ThemeMode, cx: &mut App) -> Entity<Theme> {
        let theme = cx.new(|_| Theme::new(mode));
        cx.set_global(Self(theme.clone()));
        theme
    }

    pub fn get(cx: &App) -> Entity<Theme> {
        cx.global::<Self>().0.clone()
    }
}

impl Global for ActiveTheme {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_have_distinct_semantic_colors() {
        let light = Theme::light();
        let dark = Theme::dark();

        assert_ne!(light.colors.background, dark.colors.background);
        assert_ne!(light.colors.text_primary, dark.colors.text_primary);
        assert_eq!(light.mode, ThemeMode::Light);
        assert_eq!(dark.mode, ThemeMode::Dark);
    }

    #[test]
    fn dark_theme_uses_requested_background() {
        assert_eq!(Theme::dark().colors.background, rgb(color::DARK_BACKGROUND));
    }

    #[test]
    fn dark_surfaces_are_neutral_and_layered() {
        let colors = Theme::dark().colors;

        for surface in [colors.background, colors.surface, colors.surface_elevated] {
            let brightest = surface.r.max(surface.g).max(surface.b);
            let darkest = surface.r.min(surface.g).min(surface.b);
            assert!(brightest - darkest < 0.02);
        }

        assert!(colors.background.r < colors.surface.r);
        assert!(colors.surface.r < colors.surface_elevated.r);
    }
}
