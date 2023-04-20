use gtk4 as gtk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Align, Application, ApplicationWindow, Button, FileChooserAction, FileChooserDialog, FileChooserNative,
    CompositeTemplate, };
use std::path::Path;
use std::ffi::OsStr;

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("org.example.HelloWorld")
        .build();
    
    app.connect_activate(|app| {
        // Create the main window.
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(320)
            .default_height(200)
            .title("Youtube File Storage")
            .build();
        
        let upload_button = Button::builder()
            .label("Upload File")
            .margin_top(10)
            .margin_bottom(10)
            .margin_start(10)
            .margin_end(10)
            .halign(Align::Center)
            .valign(Align::Start)
            .build();


        window.set_child(Some(&upload_button));
        // Connect the button to a callback that shows the file chooser dialog
        upload_button.connect_clicked(|_| {
            let dialog = gtk::FileChooserNative::new(
                Some("Open File"),
                Some(&Window::new(gtk::WindowType::Popup)),
                gtk::FileChooserAction::Open,
            );

            // Show the file chooser dialog and get the response
            let response = dialog.run();
    
            // Check if the user clicked the Open button
            if response == gtk::ResponseType::Accept {
                if let Some(uri) = dialog.uri() {
                    println!("Selected file: {}", uri);
                }
            };
            // Close the file chooser dialog
            dialog.hide();
        });

        window.show();
    });

    app.run()
}


