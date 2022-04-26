use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use druid::piet::TextStorage;
use druid::{Data, ImageBuf};
use image::io::Reader;
use image::{ImageError, ImageFormat};
use md5::digest::FixedOutput;
use md5::{Digest, Md5};
use tap::TapFallible;
use tracing::{debug, error};
use x11_clipboard::error::Error;
use x11_clipboard::xcb::x::{Atom, InternAtom};

const PNG_ATOM: &str = "image/png";

#[derive(Debug, Clone)]
pub struct ContentImage {
    pub raw: Arc<[u8]>,
    pub image_buf: ImageBuf,
    pub sum: [u8; 16],
}

#[derive(Debug, Clone)]
pub enum Content {
    Text(Arc<str>),
    Image(ContentImage),
}

impl From<String> for Content {
    fn from(text: String) -> Self {
        Content::Text(text.into())
    }
}

impl TryFrom<Vec<u8>> for Content {
    type Error = ImageError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let value: Arc<[u8]> = value.into();

        let image =
            Reader::with_format(io::Cursor::new(value.as_ref()), ImageFormat::Png).decode()?;

        let image_buf = ImageBuf::from_dynamic_image(image);

        let mut hasher = Md5::default();
        hasher.update(&value);
        let sum = *hasher.finalize_fixed().as_mut();

        Ok(Content::Image(ContentImage {
            raw: value,
            image_buf,
            sum,
        }))
    }
}

impl Data for Content {
    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Content::Text(text1), Content::Text(text2)) => text1 == text2,
            (Content::Image(img1), Content::Image(img2)) => img1.sum == img2.sum,

            _ => false,
        }
    }
}

pub struct Clipboard {
    x11_clipboard: x11_clipboard::Clipboard,
    png_atom: Atom,

    content_sender: Sender<Content>,
    new_content_receiver: Receiver<Content>,

    last_text: Option<Arc<str>>,
    last_image: Option<ContentImage>,
}

impl Clipboard {
    pub fn new(
        content_sender: Sender<Content>,
        new_content_receiver: Receiver<Content>,
    ) -> Result<Self> {
        let x11_clipboard = x11_clipboard::Clipboard::new()
            .tap_err(|err| error!(?err, "new x11 clipboard failed"))?;

        let png_atom = Self::get_atom(&x11_clipboard.setter.connection, PNG_ATOM)?;

        Ok(Self {
            x11_clipboard,
            png_atom,
            content_sender,
            new_content_receiver,
            last_text: None,
            last_image: None,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            if let Ok(content) = self
                .new_content_receiver
                .recv_timeout(Duration::from_millis(50))
            {
                match content {
                    Content::Text(text) => {
                        self.last_text.replace(text.clone());

                        if self
                            .x11_clipboard
                            .store(
                                self.x11_clipboard.setter.atoms.clipboard,
                                self.x11_clipboard.setter.atoms.utf8_string,
                                text.to_string(),
                            )
                            .tap_err(|err| error!(?err, %text, "store text to clipboard failed"))
                            .is_ok()
                        {
                            debug!(%text, "set text to clipboard done");
                        }
                    }

                    Content::Image(img) => {
                        let raw_img = img.raw.clone();
                        self.last_image.replace(img);

                        if self
                            .x11_clipboard
                            .store(
                                self.x11_clipboard.setter.atoms.clipboard,
                                self.png_atom,
                                raw_img.to_vec(),
                            )
                            .tap_err(|err| error!(?err, "store image to clipboard failed"))
                            .is_ok()
                        {
                            debug!("set image to clipboard done");
                        }
                    }
                }
            }

            if let Ok(Some(text)) = self.get_text() {
                if let Some(last_text) = self.last_text.as_mut() {
                    if last_text.as_str() != text {
                        *last_text = text.into();

                        let _ = self.content_sender.send(Content::Text(last_text.clone()));
                    }
                } else {
                    self.last_text.replace(text.into());

                    self.content_sender
                        .send(Content::Text(self.last_text.as_ref().unwrap().clone()))
                        .tap_err(
                            |err| error!(%err, "send content failed, maybe receiver closed"),
                        )?;
                }

                // the latest clipboard content is text, no need to get image
                continue;
            }

            if let Ok(Some(img)) = self.get_image() {
                let mut hasher = Md5::new();
                hasher.update(&img);
                let sum = *hasher.finalize_fixed().as_mut();

                if let Some(last_img) = self.last_image.as_mut() {
                    if sum != last_img.sum {
                        let image_buf = match ImageBuf::from_data(&img) {
                            Err(err) => {
                                error!(%err, "create image buf from raw image data failed");

                                continue;
                            }

                            Ok(image_buf) => image_buf,
                        };

                        *last_img = ContentImage {
                            raw: img.into(),
                            image_buf,
                            sum,
                        };

                        self.content_sender
                            .send(Content::Image(last_img.clone()))
                            .tap_err(
                                |err| error!(%err, "send content failed, maybe receiver closed"),
                            )?;

                        debug!("send image content done");
                    }
                } else {
                    let image_buf = match ImageBuf::from_data(&img) {
                        Err(err) => {
                            error!(%err, "create image buf from raw image data failed");

                            continue;
                        }

                        Ok(image_buf) => image_buf,
                    };

                    let content_image = ContentImage {
                        raw: img.into(),
                        image_buf,
                        sum,
                    };

                    self.last_image.replace(content_image.clone());

                    self.content_sender
                        .send(Content::Image(content_image))
                        .tap_err(
                            |err| error!(%err, "send content failed, maybe receiver closed"),
                        )?;

                    debug!("send image content done");
                }
            }
        }
    }

    fn get_text(&mut self) -> Result<Option<String>> {
        match self.x11_clipboard.load(
            self.x11_clipboard.setter.atoms.clipboard,
            self.x11_clipboard.setter.atoms.utf8_string,
            self.x11_clipboard.setter.atoms.property,
            Duration::from_millis(50),
        ) {
            Err(Error::UnexpectedType(_)) => Ok(None),
            Err(err) => {
                error!(?err, "get text from clipboard failed");

                Err(err.into())
            }

            Ok(text) => Ok((!text.is_empty()).then(|| String::from_utf8_lossy(&text).to_string())),
        }
    }

    fn get_image(&mut self) -> Result<Option<Vec<u8>>> {
        match self.x11_clipboard.load(
            self.x11_clipboard.setter.atoms.clipboard,
            self.png_atom,
            self.x11_clipboard.setter.atoms.property,
            Duration::from_millis(50),
        ) {
            Err(Error::UnexpectedType(_)) => Ok(None),
            Err(err) => {
                error!(?err, "get image from clipboard failed");

                Err(err.into())
            }

            Ok(img) => Ok((!img.is_empty()).then(|| img)),
        }
    }

    fn get_atom(connection: &x11_clipboard::xcb::Connection, name: &str) -> Result<Atom> {
        let req = connection.send_request(&InternAtom {
            only_if_exists: false,
            name: name.as_bytes(),
        });

        Ok(connection
            .wait_for_reply(req)
            .tap_err(|err| error!(?err, name, "get atom failed"))?
            .atom())
    }
}
