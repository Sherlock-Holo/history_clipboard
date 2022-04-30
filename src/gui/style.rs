use druid::Color;

use super::custom_button::{BorderStyle, DefaultStyle, Style, StyleSheet};

#[derive(Debug, Default, Copy, Clone)]
pub struct MyStyleSheet;

impl MyStyleSheet {
    const RADIUS: f64 = 10.0;
    fn background() -> Color {
        Color::rgb8(251, 251, 251)
    }

    fn border() -> Color {
        Color::rgb8(46, 179, 152)
    }
}

impl<T> StyleSheet<T> for MyStyleSheet {
    fn enabled(&self) -> Style<T> {
        let mut style = DefaultStyle::default().enabled();
        style.border_radius = Self::RADIUS.into();
        style.background = Some(Self::background().into());

        style
    }

    fn hovered(&self) -> Style<T> {
        let mut style = DefaultStyle::default().hovered();
        style.border = Some(BorderStyle {
            width: 2.5.into(),
            color: Self::border().into(),
        });
        style.border_radius = Self::RADIUS.into();
        style.background = Some(Self::background().into());

        style
    }

    fn pressed(&self) -> Style<T> {
        let mut style = DefaultStyle::default().pressed();
        style.border_radius = Self::RADIUS.into();
        style.background = Some(Self::background().into());

        style
    }

    fn disabled(&self) -> Style<T> {
        let mut style = DefaultStyle::default().disabled();
        style.border_radius = Self::RADIUS.into();
        style.background = Some(Self::background().into());

        style
    }
}
