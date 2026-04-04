use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Label};

fn main() {
    let app = Application::builder()
        .application_id("com.harsha.multiclip-rs")
        .build();

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {
    let label = Label::new(Some("Hello, World!"));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Hello GTK")
        .default_width(300)
        .default_height(100)
        .child(&label)
        .build();

    window.present();
}
