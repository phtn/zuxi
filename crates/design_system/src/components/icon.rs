use gpui::{Entity, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px};

use crate::{Theme, tokens::typography};

#[derive(IntoElement)]
pub struct Icon {
    glyph: SharedString,
    theme: Entity<Theme>,
    size: f32,
}

impl Icon {
    pub fn new(glyph: impl Into<SharedString>, theme: Entity<Theme>) -> Self {
        Self {
            glyph: glyph.into(),
            theme,
            size: typography::SIZE_BODY,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let color = self.theme.read(cx).colors.text_primary;

        div()
            .text_color(color)
            .text_size(px(self.size))
            .line_height(px(self.size))
            .child(self.glyph)
    }
}
