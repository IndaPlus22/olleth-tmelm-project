# Youtube Api Examples

## Authenticate user
```
let api = backend::youtubeapi::Api::new("client_secret.json").await; 
```

## Search


```
backend::youtubeapi::Api::search(&api.get_hub(), &vec!["snippet".to_string()], "dogs", 20).await;
```

## Upload
```
backend::youtubeapi::Api::single_upload(&Path::new("path/to/file"), &mut api.get_hub()).await.expect("failed uploads");
```
## Create Playlist
```
Todo!
```

## Add video to playlist
```
Todo!
```

## Remove video from playlist
```
Todo!
```