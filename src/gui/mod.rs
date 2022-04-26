use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use druid::im::Vector;
use druid::widget::{
    Container, ControllerHost, Flex, Image, Label, LineBreaking, List, ViewSwitcher,
};
use druid::{Color, Data, ExtEventSink, Key, Lens, Widget, WidgetExt};

use crate::clipboard::Content;

mod style;

pub const CONTENT_SENDER: Key<Arc<Sender<Content>>> = Key::new("history_clipboard.content_sender");

/*pub fn run() {
    let main_window = WindowDesc::new(ui_builder()).title("Clipboards");

    let image_buf = ImageBuf::from_file("1.jpeg").unwrap();

    let contents = (0..100)
        .map(|i| {
            if i % 2 == 0 {
                ClipboardContent::Text(format!("text 测试 {} 测试", i))
            } else {
                let mut hasher = Md5::default();
                hasher.update(image_buf.raw_pixels());
                let mut sum = hasher.finalize_fixed();

                ClipboardContent::Image(image_buf.clone(), *sum.as_mut())
            }
        })
        .collect();

    let clipboards = Clipboards {
        input: "".to_string(),
        contents,
    };

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(clipboards)
        .unwrap()
}*/

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
    const TEXT_COLOR: Color = Color::rgb8(0, 0, 0);

    let list = List::new(|| {
        let clickable_label = ViewSwitcher::new(
            |content: &Content, _env| content.clone(),
            |content, _clipboard_content: &Content, _env| match content {
                Content::Text(text) => {
                    let label = Label::new(text.to_string())
                        .with_text_size(20.0)
                        .with_line_break_mode(LineBreaking::Clip)
                        .with_text_color(TEXT_COLOR);

                    let label = ControllerHost::new(label, style::ButtonLabelController);

                    label.boxed()
                }

                Content::Image(content_img) => {
                    let image = Image::new(content_img.image_buf.clone());

                    let image = ControllerHost::new(image, style::ButtonLabelController);

                    image.boxed()
                }
            },
        )
        .on_click(|_ctx, content: &mut Content, env| {
            let sender: Arc<Sender<Content>> = env.get(&CONTENT_SENDER);

            let _ = sender.send(content.clone());
        })
        .padding(5.0);

        ControllerHost::new(
            Container::new(clickable_label).rounded(7.0),
            style::ContainerController,
        )
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
