mod document;
mod markdown;

use std::{env, path::PathBuf};

use design_system::{
    ActiveTheme, Theme, ThemeMode,
    components::{Button, ButtonVariant},
    tokens,
};
use document::{DocumentEntry, DocumentPreview};
use gpui::{
    AnyElement, App, AppContext, Bounds, Context, Entity, FontWeight, IntoElement, ObjectFit,
    ParentElement, Render, SharedString, Window, WindowBounds, WindowOptions, div, img, prelude::*,
    px, size,
};
use gpui_platform::application;
use markdown::{MarkdownBlock, MarkdownBlockKind};
use tempfile::TempDir;

const SIDEBAR_WIDTH: f32 = 252.0;

struct RootView {
    theme: Entity<Theme>,
    root: PathBuf,
    documents: Vec<DocumentEntry>,
    selected: Option<usize>,
    preview: DocumentPreview,
    _cache: TempDir,
}

impl RootView {
    fn new(theme: Entity<Theme>, root: PathBuf, cx: &mut Context<Self>) -> Self {
        cx.observe(&theme, |_, _, cx| cx.notify()).detach();

        let documents = document::discover(&root);
        let cache = tempfile::tempdir().expect("failed to create the PDF preview cache");
        let selected = (!documents.is_empty()).then_some(0);
        let preview = selected
            .map(|index| document::load(&documents[index], cache.path()))
            .unwrap_or(DocumentPreview::Empty);

        Self {
            theme,
            root,
            documents,
            selected,
            preview,
            _cache: cache,
        }
    }

    fn select_document(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.selected == Some(index) {
            return;
        }
        self.selected = Some(index);
        self.preview = document::load(&self.documents[index], self._cache.path());
        cx.notify();
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        let selected_path = self
            .selected
            .and_then(|index| self.documents.get(index))
            .map(|entry| entry.path.clone());
        self.documents = document::discover(&self.root);
        self.selected = selected_path
            .as_ref()
            .and_then(|path| self.documents.iter().position(|entry| &entry.path == path))
            .or((!self.documents.is_empty()).then_some(0));
        self.preview = self
            .selected
            .map(|index| document::load(&self.documents[index], self._cache.path()))
            .unwrap_or(DocumentPreview::Empty);
        cx.notify();
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = self.theme.read(cx).colors;
        let root_name = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Documents")
            .to_owned();
        let items = self
            .documents
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let is_selected = self.selected == Some(index);
                let item_background = if is_selected {
                    colors.surface_elevated
                } else {
                    colors.surface
                };
                let title = SharedString::from(entry.title.clone());
                let file_name = SharedString::from(entry.name.clone());

                div()
                    .id(("document", index))
                    .w_full()
                    .flex()
                    .items_center()
                    .gap(px(tokens::spacing::XS))
                    .px(px(tokens::spacing::XS))
                    .py(px(tokens::spacing::XXS))
                    .rounded(px(tokens::radius::MD))
                    .bg(item_background)
                    .cursor_pointer()
                    .hover(move |style| style.bg(colors.surface_elevated))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.select_document(index, cx);
                    }))
                    .child(
                        div()
                            .flex_none()
                            .w(px(32.0))
                            .rounded(px(tokens::radius::SM))
                            .bg(colors.background)
                            .text_center()
                            .text_size(px(10.0))
                            .font_weight(FontWeight(tokens::typography::WEIGHT_SEMIBOLD))
                            .text_color(colors.text_muted)
                            .child(entry.kind.label()),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(tokens::typography::SIZE_BODY))
                                    .font_weight(FontWeight(if is_selected {
                                        tokens::typography::WEIGHT_SEMIBOLD
                                    } else {
                                        tokens::typography::WEIGHT_REGULAR
                                    }))
                                    .text_color(colors.text_primary)
                                    .child(title),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(tokens::typography::SIZE_SM))
                                    .text_color(colors.text_muted)
                                    .child(file_name),
                            ),
                    )
            })
            .collect::<Vec<_>>();

        div()
            .flex_none()
            .flex()
            .flex_col()
            .w(px(SIDEBAR_WIDTH))
            .h_full()
            .border_r_1()
            .border_color(colors.border)
            .bg(colors.surface)
            .child(
                div()
                    .px(px(tokens::spacing::SM))
                    .py(px(tokens::spacing::SM))
                    .border_b_1()
                    .border_color(colors.border)
                    .child(
                        div()
                            .text_size(px(20.0))
                            .font_weight(FontWeight(tokens::typography::WEIGHT_SEMIBOLD))
                            .text_color(colors.text_primary)
                            .child("Library"),
                    )
                    .child(
                        div()
                            .mt(px(tokens::spacing::XXS))
                            .truncate()
                            .text_size(px(tokens::typography::SIZE_SM))
                            .text_color(colors.text_muted)
                            .child(root_name),
                    )
                    .child(
                        div()
                            .mt(px(tokens::spacing::XS))
                            .text_size(px(tokens::typography::SIZE_SM))
                            .text_color(colors.text_muted)
                            .child(format!("{} documents", self.documents.len())),
                    ),
            )
            .child(
                div()
                    .id("document-list")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .p(px(tokens::spacing::XXS))
                    .children(items),
            )
            .into_any_element()
    }

    fn render_toolbar(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = self.theme.read(cx).colors;
        let (title, path, kind) = self
            .selected
            .and_then(|index| self.documents.get(index))
            .map(|entry| {
                (
                    entry.title.clone(),
                    entry.relative_path.clone(),
                    entry.kind.label(),
                )
            })
            .unwrap_or_else(|| ("No document selected".into(), String::new(), "—"));
        let toggle_theme = self.theme.clone();

        div()
            .flex_none()
            .h(px(56.0))
            .flex()
            .items_center()
            .gap(px(tokens::spacing::XS))
            .px(px(tokens::spacing::SM))
            .border_b_1()
            .border_color(colors.border)
            .bg(colors.surface)
            .child(
                div()
                    .px(px(tokens::spacing::XS))
                    .rounded(px(tokens::radius::SM))
                    .bg(colors.surface_elevated)
                    .text_color(colors.text_muted)
                    .text_size(px(tokens::typography::SIZE_SM))
                    .font_weight(FontWeight(tokens::typography::WEIGHT_SEMIBOLD))
                    .child(kind),
            )
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .truncate()
                            .text_color(colors.text_primary)
                            .font_weight(FontWeight(tokens::typography::WEIGHT_SEMIBOLD))
                            .child(title),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(px(tokens::typography::SIZE_SM))
                            .text_color(colors.text_muted)
                            .child(path),
                    ),
            )
            .child(
                Button::new("reload-library", "Reload", self.theme.clone())
                    .variant(ButtonVariant::Secondary)
                    .on_click(cx.listener(|this, _, _, cx| this.reload(cx))),
            )
            .child(
                Button::new("toggle-theme", "Theme", self.theme.clone())
                    .variant(ButtonVariant::Secondary)
                    .on_click(move |_, _, cx| {
                        toggle_theme.update(cx, |theme, cx| theme.toggle(cx));
                    }),
            )
            .into_any_element()
    }

    fn render_preview(&self, cx: &mut Context<Self>) -> AnyElement {
        match &self.preview {
            DocumentPreview::Empty => self.render_empty(cx),
            DocumentPreview::Markdown(blocks) => self.render_markdown(blocks, cx),
            DocumentPreview::Pdf(preview) => self.render_pdf(preview, cx),
            DocumentPreview::Error(message) => self.render_error(message, cx),
        }
    }

    fn render_empty(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = self.theme.read(cx).colors;

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(colors.background)
            .child(
                div()
                    .max_w(px(480.0))
                    .text_center()
                    .child(
                        div()
                            .text_size(px(28.0))
                            .text_color(colors.text_primary)
                            .child("No documents found"),
                    )
                    .child(
                        div()
                            .mt(px(tokens::spacing::XS))
                            .text_color(colors.text_muted)
                            .line_height(px(tokens::typography::LINE_HEIGHT_BODY))
                            .child(
                                "Add .md or .pdf files to this folder, then click Reload. You can also pass a different folder to `cargo run -p app -- /path/to/docs`.",
                            ),
                    ),
            )
            .into_any_element()
    }

    fn render_error(&self, message: &str, cx: &mut Context<Self>) -> AnyElement {
        let colors = self.theme.read(cx).colors;

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(colors.background)
            .p(px(tokens::spacing::MD))
            .child(
                div()
                    .max_w(px(620.0))
                    .p(px(tokens::spacing::SM))
                    .border_1()
                    .border_color(colors.danger)
                    .bg(colors.surface)
                    .text_color(colors.danger)
                    .line_height(px(tokens::typography::LINE_HEIGHT_BODY))
                    .child(message.to_owned()),
            )
            .into_any_element()
    }

    fn render_markdown(&self, blocks: &[MarkdownBlock], cx: &mut Context<Self>) -> AnyElement {
        let colors = self.theme.read(cx).colors;
        let rendered_blocks = blocks
            .iter()
            .map(|block| render_markdown_block(block, colors))
            .collect::<Vec<_>>();

        div()
            .id("markdown-preview")
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .bg(colors.background)
            .child(
                div()
                    .flex_none()
                    .flex()
                    .flex_col()
                    .w_full()
                    .max_w(px(900.0))
                    .mx_auto()
                    .px(px(tokens::spacing::LG))
                    .py(px(tokens::spacing::MD))
                    .when(blocks.is_empty(), |element| {
                        element.child(
                            div()
                                .text_color(colors.text_muted)
                                .child("This Markdown file is empty."),
                        )
                    })
                    .children(rendered_blocks),
            )
            .into_any_element()
    }

    fn render_pdf(&self, preview: &document::PdfPreview, cx: &mut Context<Self>) -> AnyElement {
        let colors = self.theme.read(cx).colors;
        let page_count = preview.pages.len();
        let total_pages = preview.total_pages;
        let pages = preview
            .pages
            .iter()
            .enumerate()
            .map(|(index, page)| {
                div()
                    .w_full()
                    .max_w(px(920.0))
                    .mb(px(tokens::spacing::SM))
                    .child(
                        div()
                            .mb(px(tokens::spacing::XS))
                            .text_size(px(tokens::typography::SIZE_SM))
                            .text_color(colors.text_muted)
                            .child(format!("Page {}", index + 1)),
                    )
                    .child(
                        div()
                            .w_full()
                            .aspect_ratio(page.aspect_ratio)
                            .overflow_hidden()
                            .bg(colors.surface)
                            .shadow_lg()
                            .child(
                                img(page.path.clone())
                                    .size_full()
                                    .object_fit(ObjectFit::Contain),
                            ),
                    )
            })
            .collect::<Vec<_>>();
        let truncated = total_pages.is_some_and(|total| total > page_count);

        div()
            .id("pdf-preview")
            .size_full()
            .overflow_y_scroll()
            .bg(colors.background)
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .px(px(tokens::spacing::MD))
                    .py(px(tokens::spacing::SM))
                    .children(pages)
                    .when(truncated, |element| {
                        element.child(
                            div()
                                .mb(px(tokens::spacing::XL))
                                .p(px(tokens::spacing::SM))
                                .bg(colors.surface)
                                .text_color(colors.text_muted)
                                .child(format!(
                                    "Showing the first {page_count} of {} pages.",
                                    total_pages.unwrap_or(page_count)
                                )),
                        )
                    }),
            )
            .into_any_element()
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.theme.read(cx).colors;

        div()
            .size_full()
            .flex()
            .bg(colors.background)
            .child(self.render_sidebar(cx))
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .child(self.render_toolbar(cx))
                    .child(
                        div()
                            .min_h_0()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(self.render_preview(cx)),
                    ),
            )
    }
}

fn render_markdown_block(
    block: &MarkdownBlock,
    colors: design_system::theme::ThemeColors,
) -> AnyElement {
    match block.kind {
        MarkdownBlockKind::Heading(level) => {
            let size = match level {
                1 => 30.0,
                2 => 24.0,
                3 => 20.0,
                _ => 17.0,
            };
            div()
                .w_full()
                .mt(px(if level == 1 {
                    tokens::spacing::XXS
                } else {
                    tokens::spacing::MD
                }))
                .mb(px(tokens::spacing::XS))
                .text_size(px(size))
                .line_height(px(size + 8.0))
                .font_weight(FontWeight(tokens::typography::WEIGHT_SEMIBOLD))
                .text_color(colors.text_primary)
                .child(block.text.clone())
                .into_any_element()
        }
        MarkdownBlockKind::Paragraph => div()
            .w_full()
            .mb(px(tokens::spacing::SM))
            .text_size(px(tokens::typography::SIZE_BODY))
            .line_height(px(25.0))
            .text_color(colors.text_primary)
            .whitespace_normal()
            .child(block.text.clone())
            .into_any_element(),
        MarkdownBlockKind::Code => div()
            .w_full()
            .mb(px(tokens::spacing::SM))
            .p(px(tokens::spacing::SM))
            .border_1()
            .border_color(colors.border)
            .rounded(px(tokens::radius::MD))
            .bg(colors.surface_elevated)
            .font_family("monospace")
            .text_size(px(tokens::typography::SIZE_SM))
            .line_height(px(20.0))
            .text_color(colors.text_primary)
            .whitespace_normal()
            .child(block.text.clone())
            .into_any_element(),
        MarkdownBlockKind::Quote => div()
            .w_full()
            .mb(px(tokens::spacing::SM))
            .pl(px(tokens::spacing::SM))
            .py(px(tokens::spacing::XXS))
            .border_l_2()
            .border_color(colors.accent)
            .text_color(colors.text_muted)
            .line_height(px(tokens::typography::LINE_HEIGHT_BODY))
            .child(block.text.clone())
            .into_any_element(),
        MarkdownBlockKind::ListItem => div()
            .w_full()
            .mb(px(tokens::spacing::XXS))
            .flex()
            .gap(px(tokens::spacing::XS))
            .text_color(colors.text_primary)
            .line_height(px(tokens::typography::LINE_HEIGHT_BODY))
            .child(div().text_color(colors.accent).child("•"))
            .child(div().flex_1().child(block.text.clone()))
            .into_any_element(),
        MarkdownBlockKind::Rule => div()
            .w_full()
            .h(px(1.0))
            .my(px(tokens::spacing::MD))
            .bg(colors.border)
            .into_any_element(),
    }
}

fn library_root() -> PathBuf {
    env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("failed to determine the current directory"))
        .canonicalize()
        .expect("the document directory does not exist")
}

fn main() {
    let root = library_root();

    application().run(move |cx: &mut App| {
        ActiveTheme::init(ThemeMode::Dark, cx);

        let bounds = Bounds::centered(None, size(px(1180.0), px(760.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_, cx| {
                let theme = ActiveTheme::get(cx);
                cx.new(|cx| RootView::new(theme, root.clone(), cx))
            },
        )
        .expect("failed to open the Zuxi window");

        cx.activate(true);
    });
}
