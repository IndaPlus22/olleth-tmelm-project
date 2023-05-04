
use hyper::client::HttpConnector;
use rayon::result;
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


pub struct User {
    id: String,
    name: String,
    api: Api,
    directory: HashMap<String, DirectoryEntry>,
}

pub enum DirectoryEntry {
    Video(String),
    Playlist {
        id: String,
        videos: HashMap<String, String>,
    },
}

impl User {
    //Methods that handle the User Struct

    pub async fn new(path: &str) -> Self {
        let api = Api::new(path).await;
        let id = Api::channel_id(&api.hub()).await.expect("channel id error");
        let name = Api::channel_name(&api.hub()).await.expect("channel name error");
        let directory = User::build_directory(&api).await;

        User {
            id,
            name,
            api,
            directory,
        }
    }

    pub async fn refresh_api(&mut self, path: &str) {
        self.api = Api::new(path).await;
    }

    pub async fn update_directory(&mut self) {
        self.directory = User::build_directory(&self.api).await;
    }

}

impl User {
    //Methods that deal with video and playlist searching

    pub async fn search_directory(&self, query: &str) -> HashMap<String, String> {

        let hub = &self.api.hub();

        let mut directory: HashMap<String, String> = HashMap::new();
        let mut next_page_token: Option<String> = None;

        let result = hub.search().list(&vec!["snippet".to_owned()])
            .for_mine(true)
            .q(query)
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
                    directory.insert(video.title.unwrap(), item.id.unwrap().video_id.unwrap());
                }
    
                next_page_token = res.1.next_page_token;
                if next_page_token.is_none() {
                    return directory;
                }
            }
        };
        directory
    }

    pub async fn search_playlist(&self, playlist_id: &str) -> HashMap<String, String> {
        let mut directory = HashMap::new();
        let mut next_page_token: Option<String> = None;

        // Build the playlist items request.
        let result = self
            .api
            .hub()
            .playlist_items()
            .list(&vec!["id".to_string(), "snippet".to_string()])
            .playlist_id(playlist_id)
            .max_results(50)
            .page_token(&next_page_token.clone().unwrap_or_else(|| "".to_owned()))
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
                    let video_id = video.resource_id.unwrap().video_id.unwrap();
                    directory.insert(video.title.unwrap(), video_id);
                }
    
                next_page_token = res.1.next_page_token;
                if next_page_token.is_none() {
                    return directory;
                }
            }
        };
        directory
    }
}


impl User {
    // Methods that deal with playlist creation and handling

    pub async fn create_playlist(self, name: &str) {
        let playlist_id = Api::create_playlist(&self.api.hub(), name).await.expect("failed to create playlist!");

    }

    pub async fn remove_playlist() {
        todo!()
    }

    pub async fn add_playlistitem(self, video_id: &str, playlist_id: &str) {
        let playlistitem_id = Api::add_to_playlist(video_id, playlist_id, &self.api.hub()).await.expect("failed to add video to playlist!");
    }

    pub async fn remove_playlistitem(self, video_id: &str, playlistitem_id: &str) {
        Api::remove_from_playlist(video_id, playlistitem_id, &self.api.hub()).await.expect("failed to remove video from playlist!");
    }
}

impl User {
    // Methods that handles the upload and download operations 

    pub async fn upload(&self, file_path: &str) {
        let result = Api::upload(file_path, &self.api.hub()).await;

        match result {
            Ok(_res) => println!("Complete upload!"),
            Err(e) => println!("{}", e),
        }
    }

    pub async fn download(&self, video_id: &str, output_folder: &str)  {
        let result = Api::download(video_id, output_folder, &self.api.hub()).await;

        match result {
            Ok(_res) => println!("Complete download!"),
            Err(e) => println!("{}", e),
        }
    }
}

impl User {
    // Methods that deal with directory building

    async fn build_directory(api: &Api) -> HashMap<String, DirectoryEntry> {
        let mut directory: HashMap<String, DirectoryEntry> = HashMap::new();
        let mut next_page_token: Option<String> = None;

        loop {
            let result = api.hub().search().list(&vec!["snippet".to_owned()])
                .for_mine(true)
                .max_results(50)
                .page_token(&next_page_token.clone().unwrap_or_else(|| "".to_owned()))
                .doit().await;

            match result {
                Err(e) => match e {
                    // The Error enum provides details about what exactly happened.
                    // You can also just use its `Debug`, `Display` or `Error` traits
                    YoutubeError::HttpError(_)
                    | YoutubeError::Io(_)
                    | YoutubeError::MissingAPIKey
                    | YoutubeError::MissingToken(_)
                    | YoutubeError::Cancelled
                    | YoutubeError::UploadSizeLimitExceeded(_, _)
                    | YoutubeError::Failure(_)
                    | YoutubeError::BadRequest(_)
                    | YoutubeError::FieldClash(_)
                    | YoutubeError::JsonDecodeError(_, _) => println!("{}", e),
                },

                Ok(res) => {
                    for item in res.1.items.unwrap_or_else(Vec::new) {
                        if let Some(video_id) = &item.id.as_ref().unwrap().video_id {
                            directory.insert(item.snippet.unwrap().title.unwrap(), DirectoryEntry::Video(video_id.to_string()));
                        } else if let Some(playlist_id) = &item.id.as_ref().unwrap().playlist_id {
                            let videos = User::get_playlist_videos(api, &playlist_id).await;
                            directory.insert(item.snippet.unwrap().title.unwrap(), DirectoryEntry::Playlist { id: playlist_id.to_string(), videos });
                        }
                    }

                    next_page_token = res.1.next_page_token;
                    if next_page_token.is_none() {
                        break;
                    }
                }
            }
        }

        directory
    }

    async fn get_playlist_videos(api: &Api, playlist_id: &str) -> HashMap<String, String> {
        let hub = api.hub();
        let mut videos: HashMap<String, String> = HashMap::new();
        let mut next_page_token: Option<String> = None;
        loop {
            let result = hub.playlist_items().list(&vec!["snippet".to_owned()])
                .playlist_id(playlist_id)
                .max_results(50)
                .page_token(&next_page_token.clone().unwrap_or_else(|| "".to_owned()))
                .doit().await;
    
            match result {
                Err(e) => match e {
                    _ => println!("{}", e),
                },
                Ok(res) => {
                    for item in res.1.items.unwrap_or_else(Vec::new) {
                        if let Some(video_id) = item.snippet.as_ref().and_then(|s| s.resource_id.as_ref())
                            .and_then(|r| r.video_id.as_ref()) {
                            let title = item.snippet.as_ref().and_then(|s| s.title.as_ref())
                                .map(|s| s.to_owned()).unwrap_or_else(|| "".to_owned());
                            videos.insert(title, video_id.to_owned());
                        }
                    }
                    next_page_token = res.1.next_page_token.clone();
                    if next_page_token.is_none() {
                        break;
                    }
                }
            }
        }
    
        videos
    }
}

/// A struct representing an MP4 video file.
struct Mp4 {
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
    fn new(path: &Path, title: &str, datatype: &str, size: u32) -> Self {
        Mp4 {
            path: path.to_path_buf(),
            title: title.to_owned(),
            datatype: datatype.to_owned(),
            size,
        }
    }
}

/// A struct containing Youtube API3 client data.
struct Api {
    hub: YouTube<hyper_rustls::HttpsConnector<HttpConnector>>,
    token: yup_oauth2::AccessToken,
    expiration_time: OffsetDateTime,
}

impl Api {
    //Methods that deal with Api struct handling

    /// Constructs a new `Api` with an authenticated Youtube Data API v3 client.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the client secret file.
    async fn new(path: &str) -> Self {
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

        
        

        Api { hub, token: token.clone(), expiration_time }
    }

    /// Returns the authenticated Youtube Data API v3 hub.
    fn hub(&self) -> YouTube<hyper_rustls::HttpsConnector<HttpConnector>> {
        return self.hub.clone();
    }

    /// Returns the expiration time for the auth token.
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
}

impl Api {
    //Methods that deal with search requests

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
    async fn search(hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>) -> HashMap<String, String> {
        

        let mut video_map: HashMap<String, String> = HashMap::new();
        let mut next_page_token: Option<String> = None;

        let result = hub.search().list(&vec!["snippet".to_owned()])
            .for_mine(true)
            .max_results(50)
            .page_token(&next_page_token.clone().unwrap_or_else(|| "".to_owned()))
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
}

impl Api {
    //Methods that deal with uploading and downloading videos

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
    async fn video_upload(mp4: Mp4, hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {


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
    ///     backend::youtubeapi::Api::upload("path/to/my_file.txt", &mut api.hub()).await.expect("failed upload");
    ///     Ok(())
    /// }
    /// ```
    async fn upload(
        file_path: &str, 
        hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

        let path = Path::new(&file_path);
        
        // encode the file
        let file = FileInfo::new(&path);
        let output = Encode::encoder(Encode::new(file.clone(), (1920, 1080), 4, 4));

        // create an Mp4 instance from the encoded file
        let mp4 = Mp4::new(Path::new(&output), file.clone().name(), file.clone().datatype(), file.clone().size().try_into().unwrap());

        let result = Api::video_upload(mp4, &hub).await;
            match result {
                Ok(_res) => {
                    return Ok(());
                }
                Err(error) => {
                    return Err(error);
                }
            }
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
    async fn download(video_id: &str, output_folder: &str, hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Construct the URL of the video
        let url = format!("https://www.youtube.com/watch?v={}", video_id);
        let mut title = video_id.to_string();
        let mut datatype = ".bin".to_string();

        let result = hub.videos()
        .list(&vec!["snippet".to_string()])
        .add_id(video_id)
        .doit()
        .await
        .unwrap();

        if let Some(video) = result.1.items {
            if let Some(snippet) = video[0].snippet.clone() {
                title = snippet.title.unwrap_or_else(|| "".to_string());
                datatype = snippet.tags.and_then(|mut tags| tags.pop()).expect("tag retrieval failed!");
            }
        }

        // Construct the output file path
        let output_file = format!("{}.{}.mp4", title, datatype);
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

        match output.status.success() {
           true =>  return Ok(output_path.to_str().unwrap().to_string()),
           false => return Err("failed to download!".to_string().into()),
        }
    }
}

impl Api {
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
    async fn add_to_playlist(
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
    async fn remove_from_playlist(
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
    async fn create_playlist(
        hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>,
        name: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let playlist = Playlist {
            snippet: Some(youtube3::api::PlaylistSnippet {
                title: Some(name.to_owned()),
                ..Default::default()
            }),
            status: Some(youtube3::api::PlaylistStatus {
                privacy_status: Some("private".to_owned()),
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

impl Api {
    async fn channel_id(hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let id_request = hub.channels().list(&vec!["id".to_string()]).mine(true).doit().await.expect("error when retrieving channel id!");

        // Extract the channel ID from the response.
        
        match id_request.1.items.unwrap().first() {
            Some(channel) => {
                return Ok(channel.id.clone().unwrap());
            }
            _ => {
                return Err("channel id retrieval error".into())
            }
        }
    }

    async fn channel_name(hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let name_request = hub.channels().list(&vec!["snippet".to_string()]).mine(true).doit().await.expect("error when retrieving channel id!");

        // Extract the channel name from the response.
        match name_request.1.items.unwrap().first() {
            Some(channel) => {
                return Ok(channel.snippet.as_ref().unwrap().title.clone().unwrap());
            }
            _ => {
                return Err("channel name retrieval error".into())
            }
        }
    }        
}

