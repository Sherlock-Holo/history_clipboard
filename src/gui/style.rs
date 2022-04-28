use druid::widget::{Controller, Image, Label};
use druid::{Color, Data, Env, Event, EventCtx, PaintCtx, RenderContext, Widget};

const WHITE_COLOR: Color = Color::rgb8(255, 255, 255);
const BLACK_COLOR: Color = Color::rgb8(0, 0, 0);
const BORDER_COLOR: Color = Color::rgb8(46, 179, 152);
pub const CONTAINER_BACKGROUND: Color = Color::rgb8(210, 210, 210);

pub struct ButtonLabelController;

impl<S: Data> Controller<S, Label<S>> for ButtonLabelController {
    fn event(
        &mut self,
        child: &mut Label<S>,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut S,
        env: &Env,
    ) {
        if ctx.is_hot()
            && matches!(
                event,
                Event::MouseDown(_) | Event::MouseMove(_) | Event::MouseUp(_)
            )
        {
            child.set_text_color(WHITE_COLOR);
        } else {
            child.set_text_color(BLACK_COLOR);
        }

        ctx.request_layout();
        child.event(ctx, event, data, env)
    }
}

impl<S: Data> Controller<S, Image> for ButtonLabelController {}

pub fn container_painter<T>(pain_ctx: &mut PaintCtx, _: &T, _: &Env) {
    let rect = pain_ctx.size().to_rect();

    if pain_ctx.is_hot() {
        pain_ctx.fill(rect, &BORDER_COLOR);
    } else {
        pain_ctx.fill(rect, &CONTAINER_BACKGROUND);
    }
}
