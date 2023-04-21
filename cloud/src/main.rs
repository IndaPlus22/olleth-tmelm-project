use std::path::Path;

mod backend {
    pub mod encoder;
    pub mod file;
    pub mod youtubeapi;
}


#[tokio::main]
async fn main() {

    let api = backend::youtubeapi::Api::new("../client_secret.json").await; 

    backend::youtubeapi::Api::search("A79UdIx9aL8", &api.hub()).await;

    backend::youtubeapi::Api::upload(&Path::new("input/alpha.txt"), &api.hub()).await.expect("failed uploads");

    backend::youtubeapi::Api::download("A79UdIx9aL8", "output", &api.hub()).await;
}
