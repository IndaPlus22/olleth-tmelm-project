use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Button, FileChooserNative, ResponseType, Box, Entry, SearchEntry, ComboBoxText,
ListBox, Label, ListBoxRow};

use youtube_api::User;
use csvreader;

let clients = csvreader::create_client_secret_csv("clientsecrets").unwrap();

let mut user = User::new(clients.to_str().unwrap());

fn main() {
    // Create a new application
    let app = Application::new(Some("com.example.youtubefilestorage"), Default::default());
    app.connect_activate(build_ui);
    app.run();
}

// UI
fn build_ui(app: &Application) {
    // Create the main window
    let window = ApplicationWindow::new(app);
    window.set_title(Some("Youtube File Storage"));
    window.set_default_size(350, 70);

    // Create a vertical box container
    let vbox = Box::new(gtk4::Orientation::Vertical, 10);
    vbox.set_margin_top(10);
    vbox.set_margin_bottom(10);
    vbox.set_margin_start(10);
    vbox.set_margin_end(10);

    // Button for opening the file chooser
    let file_button = Button::with_label("Choose File to Upload");
    file_button.set_halign(gtk4::Align::Center);
    vbox.append(&file_button);

    // Create a search entry
    let search_entry = SearchEntry::new();
    search_entry.set_halign(gtk4::Align::Fill);
    vbox.append(&search_entry);

    // Create a button for sending the search query
    let search_button = Button::with_label("Search for File");
    search_button.set_halign(gtk4::Align::Center);
    vbox.append(&search_button);

    window.set_child(Some(&vbox));

    // Create an ID input entry
    let id_entry = Entry::new();
    id_entry.set_placeholder_text(Some("Enter ID"));
    id_entry.set_halign(gtk4::Align::Fill);
    vbox.append(&id_entry);

    // Create a button for downloading with specified ID and output folder
    let download_button = Button::with_label("Download");
    download_button.set_halign(gtk4::Align::Center);
    vbox.append(&download_button);

    // Create a visibility choice ComboBoxText
    let visibility_combo = ComboBoxText::new();
    visibility_combo.append(Some("private"), "Private");
    visibility_combo.append(Some("public"), "Public");
    visibility_combo.append(Some("unlisted"), "Unlisted");
    visibility_combo.set_active_id(Some("private"));  // Default choice
    vbox.append(&visibility_combo);

    // Create a button for creating a playlist
    let create_playlist_button = Button::with_label("Create Playlist");
    create_playlist_button.set_halign(gtk4::Align::Center);
    vbox.append(&create_playlist_button);

    // Create a video ID input entry
    let video_id_entry = Entry::new();
    video_id_entry.set_placeholder_text(Some("Enter Video ID"));
    video_id_entry.set_halign(gtk4::Align::Fill);
    vbox.append(&video_id_entry);

    // Create a playlist ID input entry
    let playlist_id_entry = Entry::new();
    playlist_id_entry.set_placeholder_text(Some("Enter Playlist ID"));
    playlist_id_entry.set_halign(gtk4::Align::Fill);
    vbox.append(&playlist_id_entry);

    // Create a button for moving the video to the specified playlist
    let move_video_button = Button::with_label("Add Video to Playlist");
    move_video_button.set_halign(gtk4::Align::Center);
    vbox.append(&move_video_button);

    // Create a video ID input entry
    let delete_video_id_entry = Entry::new();
    delete_video_id_entry.set_placeholder_text(Some("Enter Video ID"));
    delete_video_id_entry.set_halign(gtk4::Align::Fill);
    vbox.append(&delete_video_id_entry);

    // Create a playlist ID input entry
    let delete_playlist_id_entry = Entry::new();
    delete_playlist_id_entry.set_placeholder_text(Some("Enter Playlist ID"));
    delete_playlist_id_entry.set_halign(gtk4::Align::Fill);
    vbox.append(&delete_playlist_id_entry);

    // Create a button for moving the video to the specified playlist
    let delete_video_button = Button::with_label("Delete Video from Playlist");
    delete_video_button.set_halign(gtk4::Align::Center);
    vbox.append(&delete_video_button);

    // Create a ListBox for displaying the search results
    let search_results_list = ListBox::new();
    search_results_list.set_valign(gtk4::Align::Start);
    vbox.append(&search_results_list);

    window.set_child(Some(&vbox));

    // BUTTON ON-CLICK ACTIONS

    // Upload button
    let window_weak = window.downgrade();
    file_button.connect_clicked(move |_btn| {
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
                user.upload(file_path).await;
                
            }
            dialog.destroy();
        });

        file_chooser.show();
    });

    // Search Button
    search_button.connect_clicked(move |_btn| {
        let search_query = search_entry.text();
        println!("Search query: {}", search_query);
        
        // API call
        //let search_results = api::search(&youtube, &vec!["snippet"], search_query, 5).unwrap();
        
        // Clear the ListBox
        while let Some(child) = search_results_list.first_child() {
            search_results_list.remove(&child);
        }
    
        // Add the search results to the ListBox
        //for (key, value) in &search_results {
        //    let label = Label::new(Some(&format!("{}: {}", key, value)));
        //    let row = ListBoxRow::new();
        //    row.set_child(Some(&label));
        //    search_results_list.append(&row);
        //}

        // UI refresh
        //window.queue_draw();
    });


    // Download Button
    let window_weak = window.downgrade();
    download_button.connect_clicked(move |_btn| {
        let window = match window_weak.upgrade() {
            Some(window) => window,
            None => return,
        };

        // Get the ID input from the entry
        let id_input = id_entry.text();
        println!("ID input: {}", id_input);

        // Create a FileChooserNative dialog for selecting the output folder
        let folder_chooser = FileChooserNative::new(
            Some("Choose output folder"),
            Some(&window),
            gtk4::FileChooserAction::SelectFolder,
            Some("Select"),
            Some("Cancel"),
        );

        folder_chooser.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                let output_folder_path = dialog.file().unwrap().path().unwrap();
                println!("Output folder path: {:?}", output_folder_path);
                //Api::download(id_input, output_folder_path).await;
            }
            dialog.destroy();
        });

        folder_chooser.show();
    });

    // Create Playlist Button
    let window_weak = window.downgrade();
    create_playlist_button.connect_clicked(move |_btn| {
        let window = match window_weak.upgrade() {
            Some(window) => window,
            None => return,
        };

        // Create a FileChooserNative dialog
        let file_chooser = FileChooserNative::new(
            Some("Choose a file for the playlist"),
            Some(&window),
            gtk4::FileChooserAction::Open,
            Some("Open"),
            Some("Cancel"),
        );

        let visibility_combo_clone = visibility_combo.clone();
        file_chooser.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                let playlist_file_path = dialog.file().unwrap().path().unwrap();
                println!("Playlist file path: {:?}", playlist_file_path);

                let visibility = visibility_combo_clone.active_id().unwrap().to_string();
                println!("Playlist visibility: {}", visibility);

                // API
            }
            dialog.destroy();
        });

        file_chooser.show();
    });


    // Set Video to Playlist
    move_video_button.connect_clicked(move |_btn| {
        // Get the video ID input from the entry
        let video_id_input = video_id_entry.text();
        println!("Video ID input: {}", video_id_input);

        // Get the playlist ID input from the entry
        let playlist_id_input = playlist_id_entry.text();
        println!("Playlist ID input: {}", playlist_id_input);

        // API
    });

    // Delete Video Button
    delete_video_button.connect_clicked(move |_btn| {
        // Get the video ID input from the entry
        let delete_video_id_input = delete_video_id_entry.text();
        println!("Video ID input: {}", delete_video_id_input);

        // Get the playlist ID input from the entry
        let delete_playlist_id_input = delete_playlist_id_entry.text();
        println!("Playlist ID input: {}", delete_playlist_id_input);

        // API
    });


    window.present();
}
