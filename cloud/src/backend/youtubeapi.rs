extern crate google_youtube3 as youtube3;

use hyper::client::HttpConnector;

use youtube3::api::{Video, PlaylistItem};
use youtube3::{Error as YoutubeError, YouTube};
use yup_oauth2::{authenticator::Authenticator};
use std::path::{Path, PathBuf};
use mime::Mime;
use std::fs::{self};

use crate::backend::encoder::Encode;
use crate::backend::file::FileInfo;

pub struct Api {
    client: hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
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

        let auth =  yup_oauth2::InstalledFlowAuthenticator::builder(
            secret,
            yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        )
        .build().await.unwrap();

        let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_only()
        .enable_http1()
        .build();

        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        let hub = YouTube::new(client.clone(), auth.clone());

        Api { client, hub }
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
                description: Some((format!("{}\n{:?}\n{}", mp4.datatype, mp4.path, mp4.size)).to_owned()),

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
            let encoded_file = Encode::encoder(encoder);
            
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
        let insert_request = hub.playlist_items().insert(playlist_item).add_part("snippet")
        .doit()
        .await?;
    
        Ok(insert_request.1)
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
}
