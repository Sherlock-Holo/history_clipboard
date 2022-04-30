use std::ops::Deref;

use druid::widget::BackgroundBrush;
use druid::{
    BoxConstraints, Color, Data, Env, Event, EventCtx, KeyOrValue, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, RenderContext, RoundedRectRadii, Size, UpdateCtx, Widget,
    WidgetPod,
};
use tracing::trace;

pub struct CustomButton<T> {
    child: WidgetPod<T, Box<dyn Widget<T>>>,
    style: Box<dyn StyleSheet<T>>,
}

impl<T> CustomButton<T> {
    pub fn new<W: Widget<T> + 'static>(child: W) -> Self {
        Self {
            child: WidgetPod::new(Box::new(child)),
            style: Box::new(DefaultStyle),
        }
    }

    pub fn style<S: StyleSheet<T> + 'static>(mut self, style: S) -> Self {
        self.style = Box::new(style);

        self
    }

    pub fn set_style<S: StyleSheet<T> + 'static>(&mut self, style: S) {
        self.style = Box::new(style);
    }
}

macro_rules! style_from_ctx {
    ($ctx:expr, $style:expr) => {
        if $ctx.is_disabled() {
            $style.disabled()
        } else {
            if $ctx.is_active() {
                $style.pressed()
            } else if $ctx.is_hot() {
                $style.hovered()
            } else {
                $style.enabled()
            }
        }
    };
}

impl<T: Data> Widget<T> for CustomButton<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        match event {
            Event::MouseDown(_) => {
                if !ctx.is_disabled() {
                    ctx.set_active(true);
                    ctx.request_paint();
                    trace!("Button {:?} pressed", ctx.widget_id());
                }
            }

            Event::MouseUp(_) => {
                if ctx.is_active() && !ctx.is_disabled() {
                    ctx.request_paint();
                    trace!("Button {:?} released", ctx.widget_id());
                }
                ctx.set_active(false);
            }

            _ => {}
        }

        self.child.event(ctx, event, data, env)
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::HotChanged(_) | LifeCycle::DisabledChanged(_) = event {
            ctx.request_paint();
        }
        self.child.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        let mut style = style_from_ctx!(ctx, self.style);

        if let Some(background) = style.background.as_mut() {
            background.update(ctx, old_data, data, env);
        }

        if let Some(border) = style.border.as_ref() {
            if ctx.env_key_changed(&border.width) {
                ctx.request_layout();
            }
            if ctx.env_key_changed(&border.color) {
                ctx.request_paint();
            }
        }

        if ctx.env_key_changed(&style.border_radius) {
            ctx.request_paint();
        }

        self.child.update(ctx, data, env)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        bc.debug_check("Container");

        let style = self.style.enabled();

        // Shrink constraints by border offset
        let border_width = match &style.border {
            Some(border) => border.width.resolve(env),
            None => 0.0,
        };
        let child_bc = bc.shrink((2.0 * border_width, 2.0 * border_width));
        let size = self.child.layout(ctx, &child_bc, data, env);
        let origin = Point::new(border_width, border_width);
        self.child.set_origin(ctx, data, env, origin);

        let my_size = Size::new(
            size.width + 2.0 * border_width,
            size.height + 2.0 * border_width,
        );

        let my_insets = self.child.compute_parent_paint_insets(my_size);
        ctx.set_paint_insets(my_insets);
        let baseline_offset = self.child.baseline_offset();
        if baseline_offset > 0f64 {
            ctx.set_baseline_offset(baseline_offset + border_width);
        }

        trace!("Computed layout: size={}, insets={:?}", my_size, my_insets);
        my_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let mut style = style_from_ctx!(ctx, self.style);

        let corner_radius = style.border_radius.resolve(env);

        if let Some(background) = style.background.as_mut() {
            let panel = ctx.size().to_rounded_rect(corner_radius);

            ctx.with_save(|ctx| {
                ctx.clip(panel);
                background.paint(ctx, data, env);
            });
        }

        if let Some(border) = &style.border {
            let border_width = border.width.resolve(env);
            let border_rect = ctx
                .size()
                .to_rect()
                .inset(border_width / -2.0)
                .to_rounded_rect(corner_radius);

            ctx.stroke(border_rect, &border.color.resolve(env), border_width);
        };

        self.child.paint(ctx, data, env);
    }
}

pub struct BorderStyle {
    pub width: KeyOrValue<f64>,
    pub color: KeyOrValue<Color>,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DefaultStyle;

impl<T> StyleSheet<T> for DefaultStyle {
    fn enabled(&self) -> Style<T> {
        Style {
            background: Some(BackgroundBrush::Color(Color::rgb(0.87, 0.87, 0.87))),
            border: Some(BorderStyle {
                width: 1.0.into(),
                color: Color::rgb(0.7, 0.7, 0.7).into(),
            }),
            border_radius: 2.0.into(),
        }
    }
}

pub struct Style<T> {
    // pub shadow_offset: Vector,
    pub background: Option<BackgroundBrush<T>>,
    pub border: Option<BorderStyle>,
    pub border_radius: KeyOrValue<RoundedRectRadii>,
}

pub trait StyleSheet<T> {
    fn enabled(&self) -> Style<T>;

    fn hovered(&self) -> Style<T> {
        self.enabled()
    }

    fn pressed(&self) -> Style<T> {
        self.enabled()
    }

    fn disabled(&self) -> Style<T> {
        let mut active = self.enabled();

        let new_background = active.background.map(|background| match background {
            BackgroundBrush::Color(color) => {
                let (r, g, b, a) = color.as_rgba();

                BackgroundBrush::Color(Color::rgba(r, g, b, a * 0.5))
            }

            other => other,
        });

        active.background = new_background;

        active
    }
}

impl<T> StyleSheet<T> for Box<dyn StyleSheet<T>> {
    fn enabled(&self) -> Style<T> {
        self.deref().enabled()
    }

    fn hovered(&self) -> Style<T> {
        self.deref().hovered()
    }

    fn pressed(&self) -> Style<T> {
        self.deref().pressed()
    }

    fn disabled(&self) -> Style<T> {
        self.deref().disabled()
    }
}
