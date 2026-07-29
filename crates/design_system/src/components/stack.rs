use gpui::{AnyElement, IntoElement, ParentElement, RenderOnce, Window, div, prelude::*, px};

use crate::tokens::spacing;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StackDirection {
    Row,
    #[default]
    Column,
}

#[derive(IntoElement)]
pub struct Stack {
    direction: StackDirection,
    gap: f32,
    centered: bool,
    children: Vec<AnyElement>,
}

impl Stack {
    pub fn vertical() -> Self {
        Self {
            direction: StackDirection::Column,
            gap: spacing::MD,
            centered: false,
            children: Vec::new(),
        }
    }

    pub fn horizontal() -> Self {
        Self {
            direction: StackDirection::Row,
            ..Self::vertical()
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
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

impl RenderOnce for Stack {
    fn render(self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        div()
            .flex()
            .when(self.direction == StackDirection::Column, |element| {
                element.flex_col()
            })
            .when(self.direction == StackDirection::Row, |element| {
                element.flex_row()
            })
            .gap(px(self.gap))
            .when(self.centered, |element| {
                element.items_center().justify_center()
            })
            .children(self.children)
    }
}
