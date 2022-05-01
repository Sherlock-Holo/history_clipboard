pub mod button {
    use druid::Color;

    use super::super::custom_button::{BorderStyle, DefaultStyle, Style, StyleSheet};

    #[derive(Debug, Default, Copy, Clone)]
    pub struct CustomStyleSheet;

    impl CustomStyleSheet {
        const RADIUS: f64 = 10.0;
        fn background() -> Color {
            Color::rgb8(251, 251, 251)
        }

        fn border() -> Color {
            Color::rgb8(46, 179, 152)
        }
    }

    impl<T> StyleSheet<T> for CustomStyleSheet {
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
}

pub mod radio {
    use druid::Color;

    use super::super::custom_radio::{BorderStyle, DefaultStyle, Style, StyleSheet};

    #[derive(Debug, Default, Copy, Clone)]
    pub struct CustomStyleSheet;

    impl CustomStyleSheet {
        const RADIUS: f64 = 10.0;
        fn background() -> Color {
            Color::rgb8(251, 251, 251)
        }

        fn border() -> Color {
            Color::rgb8(46, 179, 152)
        }
    }

    impl<T> StyleSheet<T> for CustomStyleSheet {
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

        fn chosen(&self) -> Style<T> {
            self.hovered()
        }

        fn disabled(&self) -> Style<T> {
            let mut style = DefaultStyle::default().disabled();
            style.border_radius = Self::RADIUS.into();
            style.background = Some(Self::background().into());

            style
        }
    }
}
