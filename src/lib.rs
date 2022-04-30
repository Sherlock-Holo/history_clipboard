use std::sync::Arc;
use std::thread;

use anyhow::Result;
use druid::{AppLauncher, Env, Size, WindowDesc};
use tap::TapFallible;
use tracing::error;

mod clipboard;
mod gui;

pub fn run() -> Result<()> {
    let window = WindowDesc::new(gui::new_ui())
        .title("History Clipboard")
        .window_size(Size {
            width: 350.0,
            height: 500.0,
        });
    let launcher = AppLauncher::with_window(window);
    let event_sink = launcher.get_external_handle();

    let (new_content_sender, new_content_receiver) = crossbeam_channel::unbounded();
    let (content_sender, content_receiver) = crossbeam_channel::unbounded();

    // configure_env need 'static
    let new_content_sender: &'static mut _ = Box::leak(Box::new(Arc::new(new_content_sender)));

    thread::spawn(|| {
        gui::update_clipboard(event_sink, content_receiver);
    });

    let mut clipboard = clipboard::Clipboard::new(content_sender, new_content_receiver)
        .tap_err(|err| error!(%err, "create clipboard failed"))?;

    let _clipboard_thread = thread::spawn(move || clipboard.run());

    let gui_data = gui::Clipboard::new(20);

    launcher
        .configure_env(move |env: &mut Env, _state: &gui::Clipboard| {
            env.set(gui::CONTENT_SENDER, new_content_sender.clone());
        })
        .log_to_console()
        .launch(gui_data)?;

    Ok(())
}
