# Youtube Api Examples

## Authenticate user
Ger användare tillgång till att använda api:n.
```
let api = backend::youtubeapi::Api::new("client_Secret.json").await; 
```
Client_secret.json är själva authentication nyckeln som behövs för att kunna använda Youtube-api. För att få den måste du först skapa ett projekt på google cloud och sedan skapa credentials.
## Search
Hur man söker på videos genom youtube api.

```
backend::youtubeapi::Api::search(&api.get_hub(), &vec!["snippet".to_string()], "search tab", 20).await;
```
- `api.get_hub()`: Ger dig tillgång till youtube hub där all data existerar.
- `vec!["snippet".to_string()] `: En vektor som innehåller alla delar av videon du vill ta fram. snippet t.ex. innehåller videons titel, beskrivning, kanal id, m.m.
- `search tab `: Är skriver man in det man vill söka på t.ex. funny dog videos.
- `20`: Mängden resultat som ska visas.
## Upload
Hur man lägger upp en fil på youtube. 
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