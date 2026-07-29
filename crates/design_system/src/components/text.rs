use gpui::{
    Entity, FontWeight, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px,
};

use crate::{Theme, tokens::typography};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextKind {
    Small,
    #[default]
    Body,
    Heading,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextTone {
    #[default]
    Primary,
    Muted,
    Accent,
    Danger,
    OnAccent,
}

#[derive(IntoElement)]
pub struct Text {
    content: SharedString,
    theme: Entity<Theme>,
    kind: TextKind,
    tone: TextTone,
}

impl Text {
    pub fn new(content: impl Into<SharedString>, theme: Entity<Theme>) -> Self {
        Self {
            content: content.into(),
            theme,
            kind: TextKind::Body,
            tone: TextTone::Primary,
        }
    }

    pub fn small(mut self) -> Self {
        self.kind = TextKind::Small;
        self
    }

    pub fn heading(mut self) -> Self {
        self.kind = TextKind::Heading;
        self
    }

    pub fn tone(mut self, tone: TextTone) -> Self {
        self.tone = tone;
        self
    }

    pub fn muted(self) -> Self {
        self.tone(TextTone::Muted)
    }
}

impl RenderOnce for Text {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let colors = self.theme.read(cx).colors;
        let color = match self.tone {
            TextTone::Primary => colors.text_primary,
            TextTone::Muted => colors.text_muted,
            TextTone::Accent => colors.accent,
            TextTone::Danger => colors.danger,
            TextTone::OnAccent => colors.on_accent,
        };
        let (size, line_height, weight) = match self.kind {
            TextKind::Small => (
                typography::SIZE_SM,
                typography::LINE_HEIGHT_SM,
                typography::WEIGHT_REGULAR,
            ),
            TextKind::Body => (
                typography::SIZE_BODY,
                typography::LINE_HEIGHT_BODY,
                typography::WEIGHT_REGULAR,
            ),
            TextKind::Heading => (
                typography::SIZE_HEADING,
                typography::LINE_HEIGHT_HEADING,
                typography::WEIGHT_SEMIBOLD,
            ),
        };

        div()
            .text_color(color)
            .text_size(px(size))
            .line_height(px(line_height))
            .font_weight(FontWeight(weight))
            .child(self.content)
    }
}
