use design_system::{
    ActiveTheme, Theme, ThemeMode,
    components::{Button, ButtonVariant, Container, ContainerVariant, Icon, Stack, Text, TextTone},
    tokens,
};
use gpui::{
    App, AppContext, Bounds, Context, Entity, Render, Window, WindowBounds, WindowOptions,
    prelude::*, px, size,
};
use gpui_platform::application;

struct Showcase {
    theme: Entity<Theme>,
}

impl Showcase {
    fn new(theme: Entity<Theme>, cx: &mut Context<Self>) -> Self {
        cx.observe(&theme, |_, _, cx| cx.notify()).detach();
        Self { theme }
    }
}

impl Render for Showcase {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let toggle_theme = self.theme.clone();

        Container::new(self.theme.clone())
            .background()
            .full()
            .centered()
            .child(
                Container::new(self.theme.clone())
                    .variant(ContainerVariant::Elevated)
                    .padding(tokens::spacing::LG)
                    .child(
                        Stack::vertical()
                            .gap(tokens::spacing::MD)
                            .child(
                                Stack::horizontal()
                                    .gap(tokens::spacing::XS)
                                    .child(Icon::new("◆", self.theme.clone()).size(24.0))
                                    .child(
                                        Text::new("Design system showcase", self.theme.clone())
                                            .heading(),
                                    ),
                            )
                            .child(
                                Text::new(
                                    "Semantic tokens keep both themes consistent.",
                                    self.theme.clone(),
                                )
                                .muted(),
                            )
                            .child(
                                Text::new("Accent text", self.theme.clone()).tone(TextTone::Accent),
                            )
                            .child(
                                Stack::horizontal()
                                    .gap(tokens::spacing::XS)
                                    .child(
                                        Button::new(
                                            "showcase-primary",
                                            "Primary",
                                            self.theme.clone(),
                                        )
                                        .on_click(
                                            move |_, _, cx| {
                                                toggle_theme
                                                    .update(cx, |theme, cx| theme.toggle(cx));
                                            },
                                        ),
                                    )
                                    .child(
                                        Button::new(
                                            "showcase-secondary",
                                            "Secondary",
                                            self.theme.clone(),
                                        )
                                        .variant(ButtonVariant::Secondary),
                                    )
                                    .child(
                                        Button::new(
                                            "showcase-danger",
                                            "Danger",
                                            self.theme.clone(),
                                        )
                                        .variant(ButtonVariant::Danger),
                                    )
                                    .child(
                                        Button::new(
                                            "showcase-disabled",
                                            "Disabled",
                                            self.theme.clone(),
                                        )
                                        .disabled(true),
                                    ),
                            ),
                    ),
            )
    }
}

fn main() {
    application().run(|cx: &mut App| {
        ActiveTheme::init(ThemeMode::Light, cx);
        let bounds = Bounds::centered(None, size(px(760.0), px(500.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                let theme = ActiveTheme::get(cx);
                cx.new(|cx| Showcase::new(theme, cx))
            },
        )
        .expect("failed to open the design-system showcase");

        cx.activate(true);
    });
}
