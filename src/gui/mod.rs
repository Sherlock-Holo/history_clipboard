use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use custom_button::CustomButton;
use druid::im::Vector;
use druid::widget::{Button, Container, Flex, Image, Label, LineBreaking, List, ViewSwitcher};
use druid::{Color, Data, ExtEventSink, Key, Lens, Widget, WidgetExt};
use multi_type_vector::{ContentType, MultiVector};

use crate::clipboard::Content;

mod custom_button;
mod multi_type_vector;
mod style;

pub const CONTENT_SENDER: Key<Arc<Sender<Content>>> = Key::new("history_clipboard.content_sender");

#[derive(Debug, Clone, Data, Lens)]
pub struct Clipboard {
    max_size: usize,
    contents: MultiVector,
}

impl Clipboard {
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            contents: MultiVector::new(ContentType::All, Vector::new()),
        }
    }
}

pub fn new_ui() -> impl Widget<Clipboard> {
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
                        .with_text_color(TEXT_COLOR)
                        .padding(5.0);

                    CustomButton::new(label).style(style::MyStyleSheet).boxed()
                }

                Content::Image(content_img) => {
                    let image = Image::new(content_img.image_buf.clone()).padding(5.0);

                    CustomButton::new(image).style(style::MyStyleSheet).boxed()
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

    let all_button = Button::new("all")
        .on_click(|_ctx, clipboard: &mut Clipboard, _env| {
            clipboard.contents.set_content_type(ContentType::All);
        })
        .center()
        .expand_width();

    let text_button = Button::new("text")
        .on_click(|_ctx, clipboard: &mut Clipboard, _env| {
            clipboard.contents.set_content_type(ContentType::Text);
        })
        .center()
        .expand_width();

    let image_button = Button::new("image")
        .on_click(|_ctx, clipboard: &mut Clipboard, _env| {
            clipboard.contents.set_content_type(ContentType::Image);
        })
        .center()
        .expand_width();

    let top = Flex::row()
        .with_flex_child(all_button, 0.3)
        .with_flex_child(text_button, 0.3)
        .with_flex_child(image_button, 0.3);

    Flex::column()
        .with_flex_child(top, 0.1)
        .with_flex_child(list, 0.9)
        .expand_height()
        .expand_height()
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
