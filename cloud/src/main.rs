use std::path::Path;

mod backend {
    pub mod encoder;
    pub mod file;
    pub mod youtubeapi;
}


#[tokio::main]
async fn main() {


    let user = backend::youtubeapi::User::new("../client_secret.json").await;

    user.upload("path/to/file.txt").await;

    let map = user.search_directory("path/to/file.txt").await;

    let video_id = map.get("file").expect("video doesnt exist in the searched directory");

    user.download(&video_id, "path/to/folder").await;
}  
