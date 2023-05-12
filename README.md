
# Minimum Viable Product (MVP) for Cloud Storage Program

The MVP for the cloud storage program should include the following features:

## Upload 
- Ability for a user to upload a file to YouTube.
- Conversion of the uploaded file to MP4 format using ffmpeg.
- Storage of the converted file on YouTube.
## Download
- Ability for a user to download a previously uploaded file from YouTube.
- Conversion of the downloaded file from MP4 format to its original format using ffmpeg.
## Authentication
- User authentication to ensure only authorized users can access their files.
- Use of Google OAuth 2.0 to authenticate users with YouTube.
## User Interface
- A simple Graphical User Interface (GUI) for interacting with the program.

# Installing and Running the MVP
To install and run the MVP, follow these steps:

## Requirements
See External programs on how to install:
- Rust programming language
- ffmpeg
- yt-dlp
- ~~gtk4~~

## Steps
1. Clone the GitHub repository for the MVP.
2. Install Rust by following the instructions on the Rust website.
3. Install ffmpeg by following the instructions on the ffmpeg website.
4. Install yt-dlp by following the instructions on the yt-dlp website.
5. Create a Google Cloud Platform project and enable the YouTube Data API.
6. Create a client ID and client secret for the YouTube Data API.
7. Save the client ID and client secret in a file named client_secret.json in the root directory of the MVP repository.
8. Build the MVP by running cargo build in the root directory of the MVP repository.
9. Run the MVP by running cargo run in the root directory of the MVP repository.

# Next Steps
Once the MVP is complete, there are several potential features that could be added to the program, such as:

- Improved user interface (e.g., web or desktop application)
- File sharing and collaboration features
- Automatic backups and syncing of local files with cloud storage
- Encryption and security features
- Batch file processing
- User accounts
- File versioning
- Folder organization

# External Programs
## yt-dlp
yt-dlp is a command-line program to download videos from YouTube and many other video platforms. It is used in the cloud storage program to download videos from YouTube and convert them to other formats for storage.

### How it's used
In the cloud storage program, yt-dlp is used to download videos from YouTube and extract the audio and video streams for conversion by FFmpeg. The program takes in a YouTube video URL and passes it to yt-dlp as a command-line argument. The program then uses the downloaded video to create a new video file with the desired format.

### Installation
**Windows**
1. Download the latest Windows executable from the yt-dlp [release page](https://github.com/yt-dlp/yt-dlp/releases).
2. Move the executable to a location in your PATH environment variable.

**macOS**
1. Install Homebrew by following the instructions on the [Homebrew website](https://brew.sh).
2. Open a Terminal window and enter the following command to install yt-dlp: `brew install yt-dlp`.

**Linux**

yt-dlp can be installed on most Linux distributions through the package manager. For example, on Ubuntu and Debian-based systems, open a Terminal window and enter the following command: `sudo apt install yt-dlp`.

## FFmpeg
FFmpeg is a command-line program to convert audio and video files between different formats. It is used in the cloud storage program to convert downloaded YouTube videos to other formats for storage.

### How it's used
In the cloud storage program, FFmpeg is used to convert the audio and video streams extracted from a downloaded YouTube video into the desired format. The program takes in the downloaded video and passes it to FFmpeg as a command-line argument. The program then uses the converted streams to create a new video file with the desired format.

### Installation
**Windows**
1. Download the latest Windows build of FFmpeg from the official [FFmpeg website](https://ffmpeg.org/download.html#build-windows).
2. Extract the downloaded ZIP file to a directory in your PATH environment variable.

**macOS**
1. Install Homebrew by following the instructions on the [Homebrew website](https://brew.sh).
2. Open a Terminal window and enter the following command to install FFmpeg: `brew install ffmpeg`.

**Linux**

FFmpeg can be installed on most Linux distributions through the package manager. For example, on Ubuntu and Debian-based systems, open a Terminal window and enter the following command: `sudo apt install ffmpeg`.

# API functions
The Api struct provides a high-level interface to the YouTube Data API v3, allowing users to authenticate with Google and perform operations such as uploading, downloading and searching for videos. The struct has the following public functions:

##  New
The `new` function is a constructor for the `Api` struct that takes a path to a client secret file and returns a new `Api` instance. This function authenticates the user using the OAuth2 installed flow, reads the client secret file, and retrieves an access token. It also sets the expiration time for the access token and creates a new instance of the `YouTube` struct, which is used to make requests to the YouTube Data API v3.

`new(path: &str) -> Result<Self, Error>`

- `path`: A string representing the path to the client secret file.

### Example:
```rust
use youtube_api::Api;
let api = Api::new("/path/to/client_secret.json").unwrap();
```

## Search
The `search` function is an asynchronous function that takes a YouTube instance, a list of parts to include in the API response, a search query, and a maximum number of results to return. This function searches the YouTube database for videos matching the search query, and returns a hash map containing video titles and descriptions.

`search(hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>, part: &Vec<String>, query: &str, max: u32) -> HashMap<String, String>`

- `hub`: The YouTube instance, needed for making requests to the YouTube Data API v3.
- `part`: A vector of strings representing the parts to include in the API response, such as "snippet" or "contentDetails".
- `query`: A string representing the search query.
- `max`: An integer representing the maximum number of search results to return.
### Example:
```rust
use youtube_api::Api;
let api = Api::new("/path/to/client_secret.json").unwrap();
let youtube = api.get_hub();
let result = Api::search(&youtube, &vec!["snippet"], "rust programming tutorial", 5).unwrap();
println!("{:#?}", result);
```
## Upload
The `upload` function is an asynchronous function that takes a file path and a YouTube instance, and uploads the specified file to YouTube. This function encodes the file, creates a new video with a title, description, and tags based on the file metadata, and uploads the video to YouTube using the resumable upload protocol.

`upload(file_path: &Path, hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>`

- `file_path`: A Path to the file a user wants to upload.
- `hub`: The YouTube instance, needed for making requests to the YouTube Data API v3.
### Example:
```rust
use youtube_api::Api;
let api = Api::new("/path/to/client_secret.json").unwrap();
let youtube = api.get_hub();
Api::upload("/path/to/file.mp4", &youtube).unwrap();
```
Note: This function use the ffmpeg program to turn a files bytes into rgb frames. You need to install ffmpeg on your system and make sure it's in your system's path for this function to work.
`ffmpeg` is a free and open-source software that is widely used for handling multimedia files. It can be used for tasks such as converting video and audio files, resizing and cropping videos, and more.

To install `ffmpeg`, you can follow the instructions for your specific operating system. Here are a few examples:

- **Windows**: You can download a pre-built binary from the official website: https://ffmpeg.org/download.html#build-windows. Make sure to add the `bin` directory to your system's `PATH` environment variable so that the `ffmpeg` command can be run from any directory in your terminal.
- **MacOS**: You can install `ffmpeg` using Homebrew: `brew install ffmpeg`.
- **Linux**: You can install `ffmpeg` using your distribution's package manager. For example, on Ubuntu, you can run `sudo apt-get install ffmpeg`.

After installing `ffmpeg`, you should be able to use the `upload` function to upload your video to YouTube.

## Download
download function: This function downloads a YouTube video given its ID to a specified output folder. 

`download(video_id: &str, output_folder: &str)`
The function takes two arguments:

- `video_id`: a string representing the ID of the YouTube video to be downloaded.
- `output_folder`: a string representing the path to the folder where the downloaded video will be saved.

### Example: 
To download a video with ID abcdefg to the folder C:\Downloads, you can call the function like this:

```rust
api::download("abcdefg", "C:\\Downloads").await;
```

Note: This function uses the yt-dlp program to download the video. You need to install yt-dlp on your system and make sure it's in your system's path for this function to work. You can download yt-dlp from its GitHub repository: https://github.com/yt-dlp/yt-dlp.

## Add video to playlist
The `add_to_playlist` function allows you to add a video to a playlist on YouTube. The function creates a new `PlaylistItem` object, which represents the video you want to add to the playlist, and sets the `playlist_id` and `resource_id` fields of the `snippet` object to the corresponding values. It then uses the `hub` instance to call the insert method of the `playlist_items` resource, passing the `PlaylistItem` object and specifying that only the `snippet` part of the response should be returned.

`add_to_playlist(video_id: &str, playlist_id: &str, hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>) -> Result<PlaylistItem, Box<dyn std::error::Error + Send + Sync>>`

- `video_id`: The id for the YouTube Video you want to add to a specified playlist.
- `playlist_id`: The id of the playlist you want to add the video to.
- `hub`: Instance of Youtube.
### Example: 
```rust
let video_id = "abcdefg123456";
let playlist_id = "hijklmn789012";
let hub = Api::new("client_secret.json").await?.get_hub();

let playlist_item = add_to_playlist(video_id, playlist_id, &hub).await?;
println!("Video added to playlist with ID: {}", playlist_item.id.unwrap());
```
## Remove video from playlist
The `remove_from_playlist` function allows you to remove a video from a playlist on YouTube. 

`remove_from_playlist(video_id: &str, playlistitem_id: &str, hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>)`

- `video_id`: The id for the YouTube Video you want to remove to a specified playlist.
- `playlist_id`: item id representing the video in the playlist.
- `hub`: Instance of Youtube.

The function uses the `hub` instance to call the `delete` method of the `playlist_items` resource, passing the `playlistitem_id` as the argument. It then creates a new `Video` object representing the video that was removed, with the `id` field set to the `video_id`. Finally, it uses the `hub` instance to call the `update` method of the `videos` resource, passing the `Video` object and returning the updated video.
### Example: 
```rust
let video_id = "abcdefg123456";
let playlistitem_id = "pqrstuv456789";
let hub = Api::new("client_secret.json").await?.get_hub();

let video = remove_from_playlist(video_id, playlistitem_id, &hub).await?;
```

## Create playlist
The create_playlist function allows you to create a new playlist on YouTube. It takes three parameters: a hub instance of YouTube, a PathBuf representing the path of the folder or file that the playlist will be based on, and a privacy string indicating the privacy status of the playlist ("private", "unlisted", or "public").

`create_playlist(hub: &YouTube<hyper_rustls::HttpsConnector<HttpConnector>>, path: PathBuf, privacy: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>>`
- `hub`: Instance of Youtube.
- `PathBuf`: A `PathBuf` representing the path of the folder or file that the playlist will be based on.
- `privacy`: A string indicating the privacy status of the playlist ("private", "unlisted", or "public").


The function creates a new `Playlist` object, which represents the playlist you want to create, and sets the `title`, `description`, and `privacy_status` fields of the `snippet` and `status` objects to the corresponding values. It then uses the `hub` instance to call the `insert` method of the `playlists` resource, passing the `Playlist` object and returning the created playlist.
### Example: 
```rust
let folder_path = PathBuf::from("path/to/folder");
let privacy_status = "public";
let hub = Api::new("client_secret.json").await?.get_hub();

let playlist_id = create_playlist(&hub, folder_path, privacy_status).await?;
println!("Playlist created with ID: {}", playlist_id);
```