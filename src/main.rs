use gpui::{AppContext, Application, WindowOptions};
use gpui_component::Root;
use serial_port::{app::controller::AppController, view::AppView};
use std::sync::Arc;

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");
    let runtime_handle = Arc::new(runtime.handle().clone());

    let app = Application::new();
    app.run(move |cx| {
        gpui_component::init(cx);
        let runtime_handle = runtime_handle.clone();
        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), move |window, cx| {
                let (controller, event_rx) = AppController::new(runtime_handle.clone());
                let view = cx.new(|cx| AppView::new(window, cx, controller, event_rx));
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("failed to open window");
        })
        .detach();
    });
}
