use gpui::{
    AnyElement, BoxShadow, Entity, IntoElement, ParentElement, RenderOnce, Window, div, prelude::*,
    px,
};

use crate::{Theme, tokens};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ContainerVariant {
    Background,
    #[default]
    Surface,
    Elevated,
    Transparent,
}

#[derive(IntoElement)]
pub struct Container {
    theme: Entity<Theme>,
    variant: ContainerVariant,
    padding: f32,
    full: bool,
    centered: bool,
    children: Vec<AnyElement>,
}

impl Container {
    pub fn new(theme: Entity<Theme>) -> Self {
        Self {
            theme,
            variant: ContainerVariant::Surface,
            padding: tokens::spacing::MD,
            full: false,
            centered: false,
            children: Vec::new(),
        }
    }

    pub fn variant(mut self, variant: ContainerVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn background(self) -> Self {
        self.variant(ContainerVariant::Background)
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn full(mut self) -> Self {
        self.full = true;
        self
    }

    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for Container {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let colors = self.theme.read(cx).colors;
        let background = match self.variant {
            ContainerVariant::Background => colors.background,
            ContainerVariant::Surface => colors.surface,
            ContainerVariant::Elevated => colors.surface_elevated,
            ContainerVariant::Transparent => colors.background.alpha(0.0),
        };
        let elevated = self.variant == ContainerVariant::Elevated;

        div()
            .bg(background)
            .p(px(self.padding))
            .when(self.full, |element| element.size_full())
            .when(self.centered, |element| {
                element.flex().items_center().justify_center()
            })
            .when(elevated, |element| {
                element.shadow(vec![
                    BoxShadow::new(
                        px(0.0),
                        px(4.0),
                        colors.shadow.alpha(tokens::opacity::SHADOW).into(),
                    )
                    .blur_radius(px(12.0)),
                ])
            })
            .children(self.children)
    }
}
