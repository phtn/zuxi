use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MarkdownBlockKind {
    Heading(u8),
    Paragraph,
    Code,
    Quote,
    ListItem,
    Rule,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkdownBlock {
    pub kind: MarkdownBlockKind,
    pub text: String,
}

pub fn title(source: &str) -> Option<String> {
    parse(source)
        .into_iter()
        .find_map(|block| matches!(block.kind, MarkdownBlockKind::Heading(1)).then_some(block.text))
}

pub fn parse(source: &str) -> Vec<MarkdownBlock> {
    let options = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;
    let mut blocks = Vec::new();
    let mut current_kind = None;
    let mut current_text = String::new();

    for event in Parser::new_ext(source, options) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                flush(&mut blocks, &mut current_kind, &mut current_text);
                current_kind = Some(MarkdownBlockKind::Heading(heading_level(level)));
            }
            Event::Start(Tag::Paragraph) if current_kind.is_none() => {
                current_kind = Some(MarkdownBlockKind::Paragraph);
            }
            Event::Start(Tag::CodeBlock(_)) => {
                flush(&mut blocks, &mut current_kind, &mut current_text);
                current_kind = Some(MarkdownBlockKind::Code);
            }
            Event::Start(Tag::BlockQuote(_)) => {
                flush(&mut blocks, &mut current_kind, &mut current_text);
                current_kind = Some(MarkdownBlockKind::Quote);
            }
            Event::Start(Tag::Item) => {
                flush(&mut blocks, &mut current_kind, &mut current_text);
                current_kind = Some(MarkdownBlockKind::ListItem);
            }
            Event::End(TagEnd::Heading(_))
            | Event::End(TagEnd::CodeBlock)
            | Event::End(TagEnd::BlockQuote(_))
            | Event::End(TagEnd::Item) => {
                flush(&mut blocks, &mut current_kind, &mut current_text);
            }
            Event::End(TagEnd::Paragraph)
                if matches!(current_kind, Some(MarkdownBlockKind::Paragraph)) =>
            {
                flush(&mut blocks, &mut current_kind, &mut current_text);
            }
            Event::Text(text) | Event::Code(text) | Event::Html(text) | Event::InlineHtml(text) => {
                if current_kind.is_none() {
                    current_kind = Some(MarkdownBlockKind::Paragraph);
                }
                current_text.push_str(&text);
            }
            Event::SoftBreak => current_text.push(' '),
            Event::HardBreak => current_text.push('\n'),
            Event::TaskListMarker(checked) => {
                current_text.push_str(if checked { "☑ " } else { "☐ " });
            }
            Event::Rule => {
                flush(&mut blocks, &mut current_kind, &mut current_text);
                blocks.push(MarkdownBlock {
                    kind: MarkdownBlockKind::Rule,
                    text: String::new(),
                });
            }
            _ => {}
        }
    }

    flush(&mut blocks, &mut current_kind, &mut current_text);
    blocks
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn flush(
    blocks: &mut Vec<MarkdownBlock>,
    current_kind: &mut Option<MarkdownBlockKind>,
    current_text: &mut String,
) {
    if let Some(kind) = current_kind.take() {
        let text = current_text.trim().to_owned();
        if !text.is_empty() {
            blocks.push(MarkdownBlock { kind, text });
        }
    }
    current_text.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_markdown_blocks() {
        let blocks = parse(
            "# Title\n\nA paragraph with `code`.\n\n- first\n- second\n\n```rust\nfn main() {}\n```",
        );

        assert_eq!(blocks[0].kind, MarkdownBlockKind::Heading(1));
        assert_eq!(blocks[0].text, "Title");
        assert!(blocks.iter().any(|block| {
            block.kind == MarkdownBlockKind::Paragraph && block.text.contains("code")
        }));
        assert_eq!(
            blocks
                .iter()
                .filter(|block| block.kind == MarkdownBlockKind::ListItem)
                .count(),
            2
        );
        assert!(blocks.iter().any(|block| {
            block.kind == MarkdownBlockKind::Code && block.text.contains("fn main")
        }));
    }

    #[test]
    fn uses_the_first_level_one_heading_as_the_title() {
        assert_eq!(
            title("A preface.\n\n# Document title\n\nContents"),
            Some("Document title".to_owned())
        );
        assert_eq!(title("## Section only"), None);
    }
}
