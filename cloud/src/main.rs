use std::path::Path;

mod backend {
    pub mod encoder;
    pub mod file;
    pub mod youtubeapi;
}


#[tokio::main]
async fn main() {

    let api = backend::youtubeapi::Api::new("client_secret2.json").await; 

    backend::youtubeapi::Api::search(&api.get_hub(), &vec!["snippet".to_string()], "dogs", 20).await;

    backend::youtubeapi::Api::search(&api.get_hub(), &vec!["snippet".to_string()], "red bull", 20).await;

    backend::youtubeapi::Api::single_upload(&Path::new("input/alpha.txt"), &mut api.get_hub()).await.expect("failed uploads");
}
