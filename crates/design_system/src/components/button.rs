use std::rc::Rc;

use gpui::{
    ClickEvent, ElementId, Entity, IntoElement, RenderOnce, SharedString, Window, div, prelude::*,
    px,
};

use crate::{Theme, tokens};

pub type OnClick = Rc<dyn Fn(&ClickEvent, &mut Window, &mut gpui::App)>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Danger,
}

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    theme: Entity<Theme>,
    variant: ButtonVariant,
    disabled: bool,
    on_click: Option<OnClick>,
}

impl Button {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        theme: Entity<Theme>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            theme,
            variant: ButtonVariant::Primary,
            disabled: false,
            on_click: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let colors = self.theme.read(cx).colors;
        let (background, hover, text, border) = match self.variant {
            ButtonVariant::Primary => (
                colors.accent,
                colors.accent_hover,
                colors.on_accent,
                colors.accent,
            ),
            ButtonVariant::Secondary => (
                colors.surface,
                colors.surface_elevated,
                colors.text_primary,
                colors.border,
            ),
            ButtonVariant::Danger => (
                colors.danger,
                colors.danger_hover,
                colors.on_accent,
                colors.danger,
            ),
        };
        let disabled = self.disabled;
        let mut element = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .px(px(tokens::spacing::SM))
            .py(px(tokens::spacing::XXS))
            .border_1()
            .border_color(border)
            .rounded(px(tokens::radius::MD))
            .bg(background)
            .text_color(text)
            .text_size(px(tokens::typography::SIZE_BODY))
            .font_weight(gpui::FontWeight(tokens::typography::WEIGHT_MEDIUM))
            .opacity(if disabled {
                tokens::opacity::DISABLED
            } else {
                1.0
            })
            .when(!disabled, |element| {
                element.cursor_pointer().hover(move |style| style.bg(hover))
            })
            .child(self.label);

        if !disabled && let Some(on_click) = self.on_click {
            element = element.on_click(move |event, window, cx| on_click(event, window, cx));
        }

        element
    }
}
