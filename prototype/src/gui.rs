extern crate gio;
extern crate gtk;
extern crate glib;
extern crate image;

use gio::prelude::*;
use gtk::prelude::*;

use gtk::{Application, ApplicationWindow, Builder};

use std::fs::read_to_string;
use std::path::Path;
use std::fs::File;

pub(crate) fn build_ui(application: &Application) {
    // Load the UI from the Glade file
    let glade_src = read_to_string(Path::new("resources/ui.glade")).expect("Failed to read Glade file");
    let builder = Builder::from_string(&glade_src);


    // Get the main application window and set properties
    let window: ApplicationWindow = builder.get_object("main_window").expect("Couldn't get main_window");
    window.set_application(Some(application));
    window.set_title("File Uploader");

    // Show the window
    window.show_all();

    // In the build_ui function
    let upload_button: gtk::Button = builder
    .get_object("upload_button")
    .expect("Couldn't get upload_button");
    upload_button.connect_clicked(move |_| {
        // Create a FileChooserDialog for selecting the video file
        let file_chooser = gtk::FileChooserDialog::with_buttons::<gtk::Window>(
            Some("Choose File"),
            None,
            gtk::FileChooserAction::Open,
            &[
                ("_Cancel", gtk::ResponseType::Cancel),
                ("_Open", gtk::ResponseType::Accept),
            ],
        );

        if file_chooser.run() == gtk::ResponseType::Accept {
            let file = file_chooser.get_file().unwrap();
            //Send file somewhere here
        }

        file_chooser.close();
    });
}
