use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Button, FileChooserNative, ResponseType};
use youtube_api::Api;
let api = Api::new("/path/to/client_secret.json").unwrap();
let youtube = api.get_hub();

fn main() {
    // Create a new application
    let app = Application::new(Some("com.example.youtubefilestorage"), Default::default());
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    // Create the main window
    let window = ApplicationWindow::new(app);
    window.set_title(Some("Youtube File Storage"));
    window.set_default_size(350, 70);

    // Button for opening the file chooser
    let button = Button::with_label("Choose File");
    button.set_halign(gtk4::Align::Center);
    button.set_valign(gtk4::Align::Center);
    window.set_child(Some(&button));

    // Button onclick action
    let window_weak = window.downgrade();
    button.connect_clicked(move |_btn| {
        let window = match window_weak.upgrade() {
            Some(window) => window,
            None => return,
        };

        let file_chooser = FileChooserNative::new(
            Some("Choose a file"),
            Some(&window),
            gtk4::FileChooserAction::Open,
            Some("Open"),
            Some("Cancel"),
        );

        file_chooser.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                let file_path = dialog.file().unwrap().path().unwrap();
                println!("File path: {:?}", file_path); //debug
                Api::upload(file_path, &youtube).unwrap();
            }
            dialog.destroy();
        });

        file_chooser.show();
    });

    window.present();
}
