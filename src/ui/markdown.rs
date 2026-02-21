use iced::widget::{button, container, rich_text, row, span, text, Column};
use iced::{Alignment, Border, Element, Font, Length, Theme, font};

use crate::app::Message;
use crate::theme::*;

fn code_block_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(CODE_BG())),
        border: Border {
            radius: 0.0.into(),
            width: 1.0,
            color: BORDER_SUBTLE(),
        },
        ..Default::default()
    }
}

fn code_header_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(CARD_BG())),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: BORDER_SUBTLE(),
        },
        ..Default::default()
    }
}

fn copy_btn_style(_: &Theme, status: iced::widget::button::Status) -> iced::widget::button::Style {
    iced::widget::button::Style {
        background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
        text_color: match status {
            iced::widget::button::Status::Hovered => ACCENT(),
            _ => TEXT_MUTED(),
        },
        ..Default::default()
    }
}

fn blockquote_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(CARD_BG())),
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn render_markdown(content: &str) -> Element<'static, Message> {
    use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd, HeadingLevel};

    let parser = Parser::new_ext(content, Options::all());

    let mut elements: Vec<Element<'static, Message>> = Vec::new();
    let mut spans: Vec<iced::widget::text::Span<'static>> = Vec::new();
    let mut is_bold = false;
    let mut is_italic = false;
    let mut is_strikethrough = false;
    let mut in_code_block = false;
    let mut code_block_content = String::new();
    let mut code_block_lang = String::new();
    let mut heading_level: Option<HeadingLevel> = None;
    let mut list_stack: Vec<Option<u64>> = Vec::new();
    let mut in_blockquote = false;
    let mut blockquote_elements: Vec<Element<'static, Message>> = Vec::new();

    fn build_font(bold: bool, italic: bool, mono: bool) -> Font {
        if mono {
            Font {
                weight: if bold { font::Weight::Bold } else { font::Weight::Normal },
                style: if italic { font::Style::Italic } else { font::Style::Normal },
                ..Font::MONOSPACE
            }
        } else {
            Font {
                weight: if bold { font::Weight::Bold } else { font::Weight::Normal },
                style: if italic { font::Style::Italic } else { font::Style::Normal },
                ..Font::DEFAULT
            }
        }
    }

    fn flush_spans(
        spans: &mut Vec<iced::widget::text::Span<'static>>,
        elements: &mut Vec<Element<'static, Message>>,
        heading: Option<HeadingLevel>,
    ) {
        if spans.is_empty() {
            return;
        }
        let size = match heading {
            Some(HeadingLevel::H1) => FONT_MD_H1,
            Some(HeadingLevel::H2) => FONT_MD_H2,
            Some(HeadingLevel::H3) => FONT_MD_H3,
            Some(HeadingLevel::H4) => FONT_MD_H4,
            _ => FONT_BODY,
        };
        let drained: Vec<_> = std::mem::take(spans);
        elements.push(rich_text(drained).size(size).into());
    }

    for event in parser {
        match event {
            Event::Start(Tag::Paragraph) => {
                spans.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                let target = if in_blockquote { &mut blockquote_elements } else { &mut elements };
                flush_spans(&mut spans, target, None);
            }
            Event::Start(Tag::Heading { level, .. }) => {
                heading_level = Some(level);
                spans.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                flush_spans(&mut spans, &mut elements, heading_level);
                heading_level = None;
            }
            Event::Start(Tag::Strong) => {
                is_bold = true;
            }
            Event::End(TagEnd::Strong) => {
                is_bold = false;
            }
            Event::Start(Tag::Emphasis) => {
                is_italic = true;
            }
            Event::End(TagEnd::Emphasis) => {
                is_italic = false;
            }
            Event::Start(Tag::Strikethrough) => {
                is_strikethrough = true;
            }
            Event::End(TagEnd::Strikethrough) => {
                is_strikethrough = false;
            }
            Event::Start(Tag::BlockQuote(_)) => {
                let target = if in_blockquote { &mut blockquote_elements } else { &mut elements };
                flush_spans(&mut spans, target, heading_level);
                in_blockquote = true;
                blockquote_elements.clear();
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                flush_spans(&mut spans, &mut blockquote_elements, None);
                in_blockquote = false;
                let inner = Column::with_children(std::mem::take(&mut blockquote_elements)).spacing(4);
                elements.push(
                    container(
                        container(inner)
                            .padding(iced::Padding { top: 8.0, right: 12.0, bottom: 8.0, left: 16.0 })
                            .width(Length::Fill)
                            .style(blockquote_style)
                    ).width(Length::Fill).into()
                );
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let target = if in_blockquote { &mut blockquote_elements } else { &mut elements };
                flush_spans(&mut spans, target, heading_level);
                in_code_block = true;
                code_block_content.clear();
                code_block_lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                let code = std::mem::take(&mut code_block_content);
                let code = code.trim_end_matches('\n').to_string();
                let lang = std::mem::take(&mut code_block_lang);

                // Header with language label and copy button
                let lang_label = if lang.is_empty() { "code".to_string() } else { lang };
                let code_for_copy = code.clone();
                let header = container(
                    row![
                        text(lang_label).size(FONT_MICRO).color(TEXT_MUTED()).font(Font::MONOSPACE),
                        iced::widget::Space::new().width(Length::Fill),
                        button(text("Copy").size(FONT_MICRO))
                            .padding([2, 8])
                            .style(copy_btn_style)
                            .on_press(Message::CopyToClipboard(code_for_copy)),
                    ].align_y(Alignment::Center)
                )
                .padding([6, 16])
                .width(Length::Fill)
                .style(code_header_style);

                let body = container(
                    text(code)
                        .font(Font::MONOSPACE)
                        .size(FONT_SMALL)
                        .color(TEXT_BODY()),
                )
                .padding([12, 16])
                .width(Length::Fill)
                .style(code_block_style);

                let block: Element<'static, Message> = Column::new()
                    .push(header)
                    .push(body)
                    .into();

                let target = if in_blockquote { &mut blockquote_elements } else { &mut elements };
                target.push(block);
            }
            Event::Code(code) => {
                let s = span(code.into_string())
                    .font(Font::MONOSPACE)
                    .color(ACCENT());
                spans.push(s);
            }
            Event::Text(t) => {
                if in_code_block {
                    code_block_content.push_str(&t);
                } else {
                    let color = if heading_level.is_some() {
                        TEXT_HEAD()
                    } else {
                        TEXT_BODY()
                    };
                    let mut s = span(t.into_string())
                        .color(color)
                        .font(build_font(is_bold, is_italic, false));
                    if is_strikethrough {
                        s = s.strikethrough(true);
                    }
                    spans.push(s);
                }
            }
            Event::SoftBreak => {
                spans.push(span("\n"));
            }
            Event::HardBreak => {
                spans.push(span("\n"));
            }
            Event::Rule => {
                let target = if in_blockquote { &mut blockquote_elements } else { &mut elements };
                flush_spans(&mut spans, target, heading_level);
                target.push(
                    container(iced::widget::Space::new())
                        .width(Length::Fill)
                        .height(1)
                        .style(|_: &Theme| container::Style {
                            background: Some(iced::Background::Color(BORDER_DEFAULT())),
                            ..Default::default()
                        })
                        .into()
                );
            }
            Event::Start(Tag::List(ordered)) => {
                let target = if in_blockquote { &mut blockquote_elements } else { &mut elements };
                flush_spans(&mut spans, target, heading_level);
                list_stack.push(ordered);
            }
            Event::End(TagEnd::List(_)) => {
                list_stack.pop();
            }
            Event::Start(Tag::Item) => {
                spans.clear();
                let prefix = match list_stack.last() {
                    Some(Some(start)) => {
                        let n = *start;
                        if let Some(s) = list_stack.last_mut() {
                            *s = Some(n + 1);
                        }
                        format!("{n}. ")
                    }
                    _ => "\u{2022} ".to_string(),
                };
                spans.push(span(prefix).color(TEXT_MUTED()));
            }
            Event::End(TagEnd::Item) => {
                let target = if in_blockquote { &mut blockquote_elements } else { &mut elements };
                flush_spans(&mut spans, target, None);
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                let _ = dest_url;
            }
            Event::End(TagEnd::Link) => {}
            _ => {}
        }
    }

    // Flush any remaining spans
    let target = if in_blockquote { &mut blockquote_elements } else { &mut elements };
    flush_spans(&mut spans, target, heading_level);

    if elements.is_empty() {
        return text(content.to_string()).size(FONT_BODY).line_height(1.7).color(TEXT_BODY()).into();
    }

    Column::with_children(elements).spacing(8).into()
}
