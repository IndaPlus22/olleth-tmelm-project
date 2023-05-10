
mod backend {
    pub mod encoder;
    pub mod file;
    pub mod youtubeapi;
    pub mod csvreader;
}


#[tokio::main]
async fn main() {

    let clients = backend::csvreader::create_client_secret_csv("client_secrets").unwrap();

    let mut user = backend::youtubeapi::User::new(clients.to_str().unwrap()).await;


    // println!("id: {}", user.id);
    // println!("name: {}", user.name);

    //user.directory_display().await;

    // user.upload("input/beta.txt").await;

    // user.update_directory().await;

    // println!("_________________________________");

    // user.directory_display().await;

    // let video_id = map.get("file").expect("video doesnt exist in the searched directory");

    // user.download(&video_id, "path/to/folder").await;

    //backend::encoder::decoder(std::path::Path::new("output/test.txt.mp4").to_path_buf());
}  
