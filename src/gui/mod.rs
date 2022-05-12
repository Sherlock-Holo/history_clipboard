use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use custom_button::CustomButton;
use custom_radio::CustomRadio;
use druid::im::Vector;
use druid::lens::Map;
use druid::widget::{Container, Flex, Image, Label, LineBreaking, List, Svg, ViewSwitcher};
use druid::{Color, Data, ExtEventSink, Key, Lens, Widget, WidgetExt};

use crate::clipboard::Content;
use crate::gui::list_filter::ListFilter;

mod assets;
mod custom_button;
mod custom_radio;
mod list_filter;
mod style;

pub const CONTENT_SENDER: Key<Arc<Sender<Content>>> = Key::new("history_clipboard.content_sender");

#[derive(Debug, Clone, Eq, PartialEq, Data, Copy)]
enum ContentType {
    All,
    Text,
    Image,
}

#[derive(Debug, Clone, Data, Lens)]
pub struct Clipboard {
    max_size: usize,
    content_type: ContentType,
    contents: Vector<Content>,
}

impl Clipboard {
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            content_type: ContentType::All,
            contents: Vector::new(),
        }
    }
}

pub fn new_ui() -> impl Widget<Clipboard> {
    const BACKGROUND_COLOR: Color = Color::rgb8(242, 242, 242);

    let list = make_list();

    let top = make_top_ui();

    Flex::column()
        .with_flex_child(top, 0.1)
        .with_flex_child(list, 0.9)
        .expand_height()
        .expand_height()
        .background(BACKGROUND_COLOR)
}

fn make_top_ui() -> Flex<Clipboard> {
    let all_radio = CustomRadio::new(
        Svg::new(assets::ALL_SVG.parse().unwrap()).center(),
        ContentType::All,
    )
    .style(style::radio::CustomStyleSheet)
    .on_click(|_ctx, content_type: &mut ContentType, _env| {
        *content_type = ContentType::All;
    })
    .expand_width()
    .expand_height()
    .lens(Clipboard::content_type);

    let text_radio = CustomRadio::new(
        Svg::new(assets::TEXT_SVG.parse().unwrap()).center(),
        ContentType::Text,
    )
    .style(style::radio::CustomStyleSheet)
    .on_click(|_ctx, content_type: &mut ContentType, _env| {
        *content_type = ContentType::Text;
    })
    .expand_width()
    .expand_height()
    .lens(Clipboard::content_type);

    let image_radio = CustomRadio::new(
        Svg::new(assets::IMAGE_SVG.parse().unwrap()).center(),
        ContentType::Image,
    )
    .style(style::radio::CustomStyleSheet)
    .on_click(|_ctx, content_type: &mut ContentType, _env| {
        *content_type = ContentType::Image;
    })
    .expand_width()
    .expand_height()
    .lens(Clipboard::content_type);

    Flex::row()
        .with_flex_child(all_radio.padding(10.0), 0.3)
        .with_flex_child(text_radio.padding(10.0), 0.3)
        .with_flex_child(image_radio.padding(10.0), 0.3)
}

fn make_list() -> impl Widget<Clipboard> {
    const TEXT_COLOR: Color = Color::BLACK;

    let list = List::new(|| {
        let clickable_label = ViewSwitcher::new(
            |content: &Content, _env| content.clone(),
            |content, _clipboard_content: &Content, _env| match content {
                Content::Text(text) => {
                    let label = Label::new(text.to_string())
                        .with_text_size(20.0)
                        .with_line_break_mode(LineBreaking::Clip)
                        .with_text_color(TEXT_COLOR)
                        .padding(5.0);

                    CustomButton::new(label)
                        .style(style::button::CustomStyleSheet)
                        .boxed()
                }

                Content::Image(content_img) => {
                    let image = Image::new(content_img.image_buf.clone()).padding(5.0);

                    CustomButton::new(image)
                        .style(style::button::CustomStyleSheet)
                        .boxed()
                }
            },
        )
        .on_click(|_ctx, content: &mut Content, env| {
            let sender: Arc<Sender<Content>> = env.get(&CONTENT_SENDER);

            let _ = sender.send(content.clone());
        });

        Container::new(clickable_label)
            .expand_width()
            .height(100.0)
            .padding(10.0)
    });

    let list =
        ListFilter::new(
            list,
            |content: &Content, content_type: &ContentType| match content_type {
                ContentType::All => true,
                ContentType::Text => matches!(content, Content::Text(_)),
                ContentType::Image => matches!(content, Content::Image(_)),
            },
        );

    list.center()
        .expand_width()
        .scroll()
        .vertical()
        .lens(Map::new(
            |clipboard: &Clipboard| (clipboard.contents.clone(), clipboard.content_type),
            |clipboard, (contents, content_type)| {
                clipboard.contents = contents;
                clipboard.content_type = content_type;
            },
        ))
}

pub fn update_clipboard(event_sink: ExtEventSink, content_receiver: Receiver<Content>) {
    for content in content_receiver {
        event_sink.add_idle_callback(move |clipboard: &mut Clipboard| {
            clipboard.contents.push_front(content);

            while clipboard.contents.len() >= clipboard.max_size {
                clipboard.contents.pop_back();
            }
        })
    }
}
