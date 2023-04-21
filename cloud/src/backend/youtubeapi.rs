
use hyper::client::HttpConnector;
use youtube3::api::{Video, PlaylistItem, Playlist};
use youtube3::{Error as YoutubeError, YouTube};
extern crate google_youtube3 as youtube3;

use std::collections::HashMap; 
use std::path::{Path, PathBuf};
use std::process::Command;

use mime::Mime;

use time::{OffsetDateTime, UtcOffset};

use crate::backend::encoder::Encode;
use crate::backend::file::FileInfo;

/// A struct representing an authenticated Youtube Data API v3 client.
pub struct Api {
    hub: YouTube<hyper_rustls::HttpsConnector<HttpConnector>>,
    expiration_time: OffsetDateTime,
}

/// A struct representing an MP4 video file.
pub struct Mp4 {
    path: PathBuf,
    title: String,
    datatype: String,
    size: u32,
}

impl Mp4 {
    /// Constructs a new `Mp4`.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path of the MP4 video.
    /// * `title` - The title of the MP4 video.
    /// * `datatype` - The data type of the MP4 video.
    /// * `size` - The size of the MP4 video in bytes.
    pub fn new(path: &Path, title: &str, datatype: &str, size: u32) -> Self {
        Mp4 {
            path: path.to_path_buf(),
            title: title.to_owned(),
            datatype: datatype.to_owned(),
            size,
        }
    }
}

impl Api {
    /// Constructs a new `Api` with an authenticated Youtube Data API v3 client.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the client secret file.
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
        let scopes = &["https://www.googleapis.com/auth/youtube", "https://www.googleapis.com/auth/youtube.upload", "https://www.googleapis.com/auth/youtube.force-ssl"];

        let token = auth.token(scopes).await.expect("failed to retrieve token");

        let expiration_time = token.expiration_time().unwrap().to_offset(UtcOffset::from_hms(2, 0, 0).expect("could not offset to stockholm time"));

        let hub = YouTube::new(
            hyper::Client::builder().build::<_, hyper::Body>(https),
            token.token().clone().unwrap().to_string(),
        );

        
        

        Api { hub , expiration_time }
    }

    /// Returns the authenticated Youtube Data API v3 client.
    pub fn hub(&self) -> YouTube<hyper_rustls::HttpsConnector<HttpConnector>> {
        return self.hub.clone();
    }

    /// Returns the authenticated Youtube Data API v3 client.
    pub fn expiration_time(&self) -> OffsetDateTime {
        return self.expiration_time;
    }

    /// Converts a given number of bytes into
    fn convert_bytes(bytes: u64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;
    
        if bytes < KB as u64 {
            format!("{} bytes", bytes)
        } else if bytes < MB as u64 {
            format!("{:.2} KB", bytes as f64 / KB)
        } else if bytes < GB as u64 {
            format!("{:.2} MB", bytes as f64 / MB)
        } else {
            format!("{:.2} GB", bytes as f64 / GB)
        }
    }

    /// Searches for videos on YouTube based on a query and returns a hashmap of the video titles and ids.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The id of the channel you want to search through.
    /// * `hub` - A reference to the YouTube API client.
    /// 
    /// # Returns
    /// 
    /// A hashmap containing the titles and ids of the videos found on YouTube that match the search query.
    pub async fn search(channel_id: &str, hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>) -> HashMap<String, String> {
        

        let mut video_map: HashMap<String, String> = HashMap::new();
        let mut next_page_token: Option<String> = None;

        let result = hub.search().list(&vec!["snippet".to_owned()])
            .channel_id(channel_id)
            .max_results(50)
            .page_token(&next_page_token.clone().unwrap())
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
                    video_map.insert(video.title.unwrap(), item.id.unwrap().video_id.unwrap());
                }
    
                next_page_token = res.1.next_page_token;
                if next_page_token.is_none() {
                    return video_map;
                }
            }
        };
        video_map
    }

    /// This function uploads a video to YouTube and returns the video ID on success, or an error on failure.
    ///
    /// # Arguments
    ///
    /// * `mp4` - An `Mp4` object containing information about the video to upload.
    /// * `hub` - A reference to a `YouTube` object.
    ///
    /// # Returns
    ///
    /// A `Result` containing the video ID on success, or an error on failure.
    async fn upload_function(mp4: Mp4, hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {


        // create a new video
        let video = youtube3::api::Video {
            snippet: Some(youtube3::api::VideoSnippet {
                title: Some(Path::new(&mp4.title).with_extension("").display().to_string()),
                description: Some((format!("name: {}\ndatatype: {}\nsize: {}", mp4.title, mp4.datatype, Self::convert_bytes(mp4.size as u64))).to_owned()),
                tags: Some(vec![mp4.title, mp4.datatype, Self::convert_bytes(mp4.size as u64)]),
                ..Default::default()
            }),
            status: Some(youtube3::api::VideoStatus {
                privacy_status: Some("public".to_owned()),
                self_declared_made_for_kids: Some(true),
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

    /// Uploads a video file to YouTube using the given YouTube API client.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path of the video file to upload.
    /// * `hub` - The YouTube API client to use for the upload.
    ///
    /// # Errors
    ///
    /// Returns an error if the upload fails.
    ///
    /// # Example
    ///
    /// ```
    /// use youtube_upload::YouTube;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ///     let api = backend::youtubeapi::Api::new("my_api_key").await; 
    ///     backend::youtubeapi::Api::upload("path/to/my_file.txt", &mut api.hub()).await.expect("failed uploads");
    ///     Ok(())
    /// }
    /// ```
    pub async fn upload(
        file_path: &str, 
        hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

        let path = Path::new(&file_path);
        
        // encode the file
        let file = FileInfo::new(&path);
        let output = Encode::encoder(Encode::new(file.clone(), (1920, 1080), 4, 4));

        // create an Mp4 instance from the encoded file
        let mp4 = Mp4::new(Path::new(&output), file.clone().name(), file.clone().datatype(), file.clone().size().try_into().unwrap());

        let result = Self::upload_function(mp4, &hub).await;
            match result {
                Ok(video_id) => {
                    println!("Video uploaded: https://www.youtube.com/watch?v={}", video_id);
                }
                Err(error) => {
                    eprintln!("Error uploading video: {:?}", error.to_string());
                }
            }

        Ok(())
    }

    /// Downloads a YouTube video specified by its ID to the given output folder.
    ///
    /// # Arguments
    ///
    /// * `video_id` - A string slice containing the ID of the YouTube video to download.
    /// * `output_folder` - A string slice containing the path of the folder to save the downloaded video to.
    /// * `hub` - A reference to a `YouTube` object.
    ///
    /// # Example
    ///
    /// ```
    /// use youtubeapi::Api;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let api = backend::youtubeapi::Api::new("my_api_key").await; 
    ///     let video_id = "dQw4w9WgXcQ";
    ///     let output_folder = "C:/Users/Username/Videos";
    ///     backend::youtubeapi::Api::download(video_id, output_folder, &api.hub());
    /// }
    /// ```
    pub async fn download(video_id: &str, output_folder: &str, hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>) {
        // Construct the URL of the video
        let url = format!("https://www.youtube.com/watch?v={}", video_id);
    
        // Construct the output file path
        let output_file = format!("{}.mp4", video_id);
        let output_path = PathBuf::from(output_folder).join(output_file);

        // Use yt-dlp to download the video to the specified output path
        let output = Command::new("yt-dlp")
            .arg(url)
            .arg("-f")
            .arg("bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best")
            .arg("-o")
            .arg(output_path.to_str().unwrap())
            .output()
            .unwrap();

        if output.status.success() {
            println!("Video downloaded successfully to {}!", output_path.to_str().unwrap());
            //Todo! Delete video from youtube after its been downloaded
            
        } else {
            println!("Error downloading video: {:?}", output.stderr);
        }
    }
    
    ///
    /// # Arguments
    ///
    /// * `video_id` - A string slice representing the ID of the YouTube video to add to the playlist.
    /// * `playlist_id` - A string slice representing the ID of the playlist to which the video should be added.
    /// * `hub` - A reference to a `YouTube<hyper_rustls::HttpsConnector<HttpConnector>>` instance that will be used to make the API request.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `PlaylistItem` if the video was added successfully, or an error if the API request failed.
    ///
    /// # Errors
    ///
    /// Returns a `Box<dyn std::error::Error + Send + Sync>` if the API request fails.
    ///
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

    /// Removes a video from a playlist given its video ID and playlist item ID.
    ///
    /// # Arguments
    ///
    /// * `video_id` - A string slice representing the ID of the video to remove.
    /// * `playlistitem_id` - A string slice representing the ID of the playlist item to remove.
    /// * `hub` - A reference to a `YouTube` client instance.
    ///
    /// # Returns
    ///
    /// Returns a `Video` struct wrapped in a `Result` indicating whether the operation was successful.
    /// If the operation was successful, the `Video` struct contains information about the removed video.
    /// If an error occurred, the `Box<dyn std::error::Error + Send + Sync>` error is returned.
    ///
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

    /// Creates a new playlist on YouTube with the specified title, description, and privacy setting.
    ///
    /// # Arguments
    ///
    /// * `hub` - A reference to a `YouTube` client instance.
    /// * `path` - A `PathBuf` representing the path of the directory to use as the title of the playlist.
    /// * `privacy` - A string slice representing the privacy setting of the playlist.
    ///
    /// # Returns
    ///
    /// Returns a string wrapped in a `Result` indicating whether the operation was successful.
    /// If the operation was successful, the string contains the ID of the newly created playlist.
    /// If an error occurred, the `Box<dyn std::error::Error + Send + Sync>` error is returned.
    ///
    pub async fn create_playlist(
        hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>,
        path: PathBuf,
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
                return Ok(id);

            }
            None => Err("Failed to create playlist".into())
        }
    }        
}

/*
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
            let output = Encode::encoder(Encode::new(file.clone(), (1280, 720), 4, 4));
            
            tokio::spawn(async move {

                // create an Mp4 instance from the encoded file
                let mp4 = Mp4::new(Path::new(&output), file.clone().name(), file.clone().datatype(), file.clone().size().try_into().unwrap());

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

        let cut_path = Path::new(path.file_name().unwrap());
    
        let mut playlist_map: HashMap<(PathBuf, String), Vec<PathBuf>> = HashMap::new(); // keep track of playlists and their parent directories
        let mut files: Vec<_> = Vec::new();

        // collect a list of all files in the directory and its subdirectories 
        for entry in WalkDir::new(&cut_path) {
            let entry = entry?;
            if entry.path().is_file() {
                files.push(entry);
            } else if entry.path().is_dir() {
                //let playlist_name = entry.file_name().to_string_lossy().to_string();
                let parent_path = entry.path().strip_prefix(&cut_path).unwrap().parent().map_or_else(|| String::from(""), |p| p.to_string_lossy().to_string());
                playlist_map.insert((entry.path().strip_prefix(&cut_path).unwrap().to_path_buf(), parent_path), Vec::new());
            }
        }

        // add each file to the appropriate playlist
        for entry in files {
            let file_path = entry.path().strip_prefix(&cut_path).unwrap();
            let dir_name = file_path.parent().map_or_else(|| "".into(), |p| p.to_path_buf());

            println!("{:?}",  &dir_name);
            println!("{:?}",  &dir_name.to_string_lossy().to_string());
            let parent_path = dir_name.parent().and_then(|p| p.file_name()).map_or("".to_string(), |n| n.to_string_lossy().to_string());
            if let Some(playlist) = playlist_map.get_mut(&(dir_name.clone(), parent_path)) {
                 playlist.push(file_path.to_path_buf());
            }
        }
        
        //run the upload tasks and playlist creation tasks concurrently
        let mut tasks = Vec::new();
        for ((playlist_name, parent_path), playlist_files) in playlist_map {
            if !playlist_files.is_empty() {
                let path = cut_path.clone().to_path_buf();
                let hub = hub.clone(); // clone the hub to move into the closure
                tasks.push(tokio::spawn( async move {

                    let mut playlist_id = "".to_string();
                    if !playlist_name.to_string_lossy().to_string().is_empty() {
                        playlist_id = Api::create_playlist(&hub, playlist_name, parent_path, "public").await.unwrap();
                        println!("Playlist created: https://www.youtube.com/playlist?list={:?}", playlist_id);
                    }

                    for file_path in playlist_files {
                        let file = FileInfo::new(&path.join(&file_path));
                        let encoder = Encode::new(file.clone(), (1280, 720), 4, 4);
                        let mp4_path = Encode::encoder(encoder);
                        let mp4 = Mp4::new(Path::new(&mp4_path), file.clone().name(), file.clone().datatype(), file.clone().size().try_into().unwrap());
                        let video_id = Api::upload(mp4, &hub).await.expect("upload video error");
                        println!("Video uploaded: https://www.youtube.com/watch?v={:?}", video_id);
                        if !playlist_id.is_empty() {
                            Api::add_to_playlist(&video_id, &playlist_id, &hub).await.expect("error when adding to playlist");
                            println!("Video added to playlist: https://www.youtube.com/playlist?list={:?}", playlist_id);
                        }
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
 */