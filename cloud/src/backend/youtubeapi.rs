extern crate google_youtube3 as youtube3;

use hyper::client::HttpConnector;

use youtube3::api::{Video, PlaylistItem, Playlist};
use youtube3::{Error as YoutubeError, YouTube};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use mime::Mime;
use std::fs::{self};

use walkdir::WalkDir;

use crate::backend::encoder::Encode;
use crate::backend::file::FileInfo;

pub struct Api {
    hub: YouTube<hyper_rustls::HttpsConnector<HttpConnector>>,
}

struct Mp4 {
    path: PathBuf,
    title: String,
    datatype: String,
    size: u32,
}

impl Mp4 {
    fn new(path: &Path, title: &str, datatype: &str, size: u32) -> Self {
        Mp4 {
            path: path.to_path_buf(),
            title: title.to_owned(),
            datatype: datatype.to_owned(),
            size,
        }
    }
}

impl Api {
    pub async fn new(path: &str) -> Self {
        let secret =  yup_oauth2::read_application_secret(path)
        .await
        .expect("client_secret.json");

        let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
            secret,
            yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        )
        .build()
        .await
        .expect("failed to create authenticator");

        let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_only()
        .enable_http1()
        .build();

        // Define the scopes your application requires access to
        let scopes = &["https://www.googleapis.com/auth/youtube.upload", "https://www.googleapis.com/auth/youtube.force-ssl"];

        let token = auth.token(scopes).await.expect("failed to retrieve token").token().clone().unwrap().to_string();

        let hub = YouTube::new(
            hyper::Client::builder().build::<_, hyper::Body>(https),
            token,
        );
        

        Api { hub }
    }

    pub fn get_hub(&self) -> YouTube<hyper_rustls::HttpsConnector<HttpConnector>> {
        return self.hub.clone();
    }


    pub async fn search(hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>, part: &Vec<String>, query: &str, max: u32) {
        let result = hub.search().list(part)
            .q(query)
            .max_results(max)
             .doit().await;

        match result {
            Err(e) => match e {
                // The Error enum provides details about what exactly happened.
                // You can also just use its `Debug`, `Display` or `Error` traits
                YoutubeError::HttpError(_)
                |YoutubeError::Io(_)
                |YoutubeError::MissingAPIKey
                |YoutubeError::MissingToken(_)
                |YoutubeError::Cancelled
                |YoutubeError::UploadSizeLimitExceeded(_, _)
                |YoutubeError::Failure(_)
                |YoutubeError::BadRequest(_)
                |YoutubeError::FieldClash(_)
                |YoutubeError::JsonDecodeError(_, _) => println!("{}", e),
            },
            
            Ok(res) => { 
                for item in res.1.items.unwrap_or_else(Vec::new) {
                    let video = item.snippet.unwrap();
                    println!("\nNew Video ---------------------------");
                    println!("Title: {}", video.title.unwrap());
                    println!("Date: {}", video.published_at.unwrap());
                    println!("-------------------------------------");
                    println!("Channel: {}", video.channel_title.unwrap());
                    println!("id: {}", video.channel_id.unwrap());
                    println!("-------------------------------------");
                    println!("Description {}", video.description.unwrap());      
                }
            },
        }
    }

    async fn upload(mp4: Mp4, hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {

        // create a new video
        let video = youtube3::api::Video {
            snippet: Some(youtube3::api::VideoSnippet {
                title: Some(mp4.title.to_owned()),
                description: Some((format!("datatype: {}\npath: {:?}\n size: {}", mp4.datatype, mp4.path, mp4.size)).to_owned()),

                ..Default::default()
            }),
            status: Some(youtube3::api::VideoStatus {
                privacy_status: Some("private".to_owned()),
                ..Default::default()
            }),
            ..Default::default()

        };

        // read the video file
        let video_data = std::fs::read(mp4.path)?;
        let mime_type: Mime = "video/mp4".parse()?;
        let video_data_cursor = &mut std::io::Cursor::new(video_data);

        // upload the video
        let insert_request = hub.videos()
            .insert(video)
            .upload_resumable(
                video_data_cursor,
                mime_type,
            );
        let (_, response) = insert_request.await?;
        Ok(response.id.expect("missing video ID"))
    }

    pub async fn multiple_uploads(
        dir_path: &Path, 
        hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

        
        // collect a list of all files in the directory
        let files: Vec<_> = fs::read_dir(dir_path)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .collect();

        // run the upload tasks concurrently
        let mut upload_tasks = files.into_iter().map(|entry| {
            let file_path = entry.path();
            let hub = hub.clone();

            // encode the file
            let file = FileInfo::new(&file_path);
            let encoder = Encode::new(file.clone(), (1280, 720), 4, 4);
            Encode::encoder(encoder);
            
            tokio::spawn(async move {

                // create an Mp4 instance from the encoded file
                let mp4 = Mp4::new(&file_path, file.clone().name(), file.clone().datatype(), file.clone().size().try_into().unwrap());

                let result = Self::upload(mp4, &hub).await;
                match result {
                    Ok(video_id) => {
                        println!("Video uploaded: https://www.youtube.com/watch?v={}", video_id);
                    }
                    Err(error) => {
                        eprintln!("Error uploading video: {:?}", error.to_string());
                    }
                }
            })
        });

        // wait for all tasks to complete
        while let Some(task) = upload_tasks.next() {
            task.await?;
        }

        Ok(())
    }

    pub async fn multiple_uploads2<'a>(
        path: &Path,
        hub: &'a mut YouTube<hyper_rustls::HttpsConnector<HttpConnector>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

        
        //let mut tasks = Vec::new();
    
        let mut playlist_map: HashMap<(PathBuf, String), Vec<PathBuf>> = HashMap::new(); // keep track of playlists and their parent directories
        let mut files: Vec<_> = Vec::new();

        // collect a list of all files in the directory and its subdirectories 
        for entry in WalkDir::new(Path::new(&path.file_name().unwrap())) {
            let entry = entry?;
            if entry.path().is_file() {
                files.push(entry);
            } else if entry.path().is_dir() {
                //let playlist_name = entry.file_name().to_string_lossy().to_string();
                let parent_path = entry.path().parent().unwrap().to_string_lossy().to_string();
                playlist_map.insert((entry.path().to_path_buf(), parent_path), Vec::new());
            }
        }

        // add each file to the appropriate playlist
        for entry in files {
            let file_path = entry.path();
            let dir_name = file_path.parent().unwrap().to_path_buf();
            let parent_path = file_path.parent().unwrap().parent().unwrap().to_string_lossy().to_string();
            if let Some(playlist) = playlist_map.get_mut(&(dir_name, parent_path)) {
                playlist.push(file_path.to_path_buf());
            }
        }

        // run the upload tasks and playlist creation tasks concurrently
        let mut tasks = Vec::new();
        for ((playlist_name, parent_path), playlist_files) in playlist_map {
            if !playlist_files.is_empty() {
                let hub = hub.clone(); // clone the hub to move into the closure
                tasks.push(tokio::spawn( async move {
                    let playlist_id = Api::create_playlist(&hub, playlist_name, parent_path, "public").await.unwrap();
                    println!("Playlist created: https://www.youtube.com/playlist?list={:?}", playlist_id);

                    for file_path in playlist_files {
                        let file = FileInfo::new(&file_path);
                        let encoder = Encode::new(file.clone(), (1280, 720), 4, 4);
                        Encode::encoder(encoder);
                        //let mp4 = Mp4::new(&file_path, file.clone().name(), file.clone().datatype(), file.clone().size().try_into().unwrap());
                        // let video_id = Api::upload(mp4, &hub).await.expect("upload video error");
                        // println!("Video uploaded: https://www.youtube.com/watch?v={:?}", video_id);
                        // Api::add_to_playlist(&video_id, &playlist_id, &hub).await.expect("error when adding to playlist");
                        // println!("Video added to playlist: https://www.youtube.com/playlist?list={:?}", playlist_id);
                    }
                }));
            }
        }

        // wait for all tasks to complete
        for task in tasks {
            task.await?;
        }
    
        Ok(())
    }

    pub async fn add_to_playlist(
        video_id: &str,
        playlist_id: &str,
        hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>,
    ) -> Result<PlaylistItem, Box<dyn std::error::Error + Send + Sync>> {
        let playlist_item = youtube3::api::PlaylistItem {
            snippet: Some(youtube3::api::PlaylistItemSnippet {
                playlist_id: Some(playlist_id.to_owned()),
                resource_id: Some(youtube3::api::ResourceId {
                    kind: Some("youtube#video".to_owned()),
                    video_id: Some(video_id.to_owned()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (_, insert_request) = hub.playlist_items().insert(playlist_item).add_part("snippet")
        .doit()
        .await?;
    
        Ok(insert_request)
    }

    pub async fn remove_from_playlist(
        video_id: &str,
        playlistitem_id: &str,
        hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>,
    ) -> Result<Video, Box<dyn std::error::Error + Send + Sync>> {

        hub.playlist_items().delete(playlistitem_id).doit().await?;

        let video = Video {
            id: Some(video_id.to_string()),
            ..Default::default()
        };

        let update_request = hub.videos().update(video).doit();
        let (_, videos_update_response) = update_request.await?;
        Ok(videos_update_response)
    }


    pub async fn create_playlist(
        hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>,
        path: PathBuf,
        parent: String,
        privacy: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let playlist = Playlist {
            snippet: Some(youtube3::api::PlaylistSnippet {
                title: Some(path.file_name().unwrap_or_default().to_string_lossy().to_string()),
                description: Some(path.to_str().unwrap().to_owned()),
                ..Default::default()
            }),
            status: Some(youtube3::api::PlaylistStatus {
                privacy_status: Some(privacy.to_owned()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let request = hub.playlists().insert(playlist);
        let (_, child_p) = request.doit().await?;

        match child_p.id {
            Some(id) => {
                println!("Playlist created successfully with ID: {}", id);
        
                let response = hub
                    .playlists()
                    .list(&vec!["id".to_string()])
                    .mine(true)
                    .add_part("snippet")
                    .max_results(50u32)
                    .doit()
                    .await?;
        
                let parent_items = response.1.items.unwrap_or_default();
                for parent_p in parent_items {
                    if parent_p.snippet.is_some() && parent_p.snippet.as_ref().unwrap().title == Some(parent.to_owned()) {
                        Api::add_playlist(hub,&parent_p.id.unwrap(), &id).await?;
                        println!("added playlists");
                    }
                }

                return Ok(id);

            }
            None => Err("Failed to create playlist".into())
        }
    }     
    
    pub async fn add_playlist(
        hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>,
        parent_playlist_id: &str,
        child_playlist_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = hub.playlist_items().insert(PlaylistItem {
            snippet: Some(youtube3::api::PlaylistItemSnippet {
                playlist_id: Some(parent_playlist_id.to_owned()),
                resource_id: Some(youtube3::api::ResourceId {
                    kind: Some("youtube#playlist".to_owned()),
                    video_id: None,
                    playlist_id: Some(child_playlist_id.to_owned()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        });
    
        request.doit().await?;
    
        Ok(())
    }
}
