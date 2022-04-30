use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use custom_button::CustomButton;
use druid::im::Vector;
use druid::widget::{Container, Flex, Image, Label, LineBreaking, List, ViewSwitcher};
use druid::{Color, Data, ExtEventSink, Key, Lens, Widget, WidgetExt};

use crate::clipboard::Content;

mod custom_button;
mod style;

pub const CONTENT_SENDER: Key<Arc<Sender<Content>>> = Key::new("history_clipboard.content_sender");

#[derive(Debug, Clone, Data, Lens)]
pub struct Clipboard {
    max_size: usize,
    contents: Vector<Content>,
}

impl Clipboard {
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            contents: Default::default(),
        }
    }
}

pub fn ui_builder() -> impl Widget<Clipboard> {
    const BACKGROUND_COLOR: Color = Color::rgb8(242, 242, 242);
    const TEXT_COLOR: Color = Color::BLACK;

    let list = List::new(|| {
        let clickable_label = ViewSwitcher::new(
            |content: &Content, _env| content.clone(),
            |content, _clipboard_content: &Content, _env| match content {
                Content::Text(text) => {
                    let label = Label::new(text.to_string())
                        .with_text_size(20.0)
                        .with_line_break_mode(LineBreaking::Clip)
                        .with_text_color(TEXT_COLOR);

                    CustomButton::new(label)
                        .style(style::MyStyleSheet)
                        .padding(5.0)
                        .boxed()
                }

                Content::Image(content_img) => {
                    let image = Image::new(content_img.image_buf.clone());

                    CustomButton::new(image)
                        .style(style::MyStyleSheet)
                        .padding(5.0)
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
    })
    .center()
    .expand_width()
    .scroll()
    .vertical()
    .lens(Clipboard::contents);

    let flex = Flex::column().with_flex_child(list, 1.0);

    Container::new(flex)
        .expand_height()
        .expand_width()
        .background(BACKGROUND_COLOR)
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
