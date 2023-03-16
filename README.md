# Podgrab V2

Podgrab is a self-hosted podcast manager. 
It is a web app that lets you download podcasts and listen to them online.
It is written in Rust and uses React for the frontend.


# Getting Started

## Docker

### Docker-Compose

```yaml
version: '3'
services:
  podgrabv2:
    image: samuel19982/podgrabv2:latest
    ports:
      - "80:8000"
    volumes:
      - podgrab-podcasts:/app/podcasts
      - podgrab-db:/app/podcast.db
    environment:
      - POLLING_INTERVAL=60
      - SERVER_URL=http://<url to the server>
```

# Environment Variables

| Variable         | Description                                   | Default               |
|------------------|-----------------------------------------------|-----------------------|
| POLLING_INTERVAL | Interval in minutes to check for new episodes | 60                    |
| SERVER_URL       | URL of the server                             | http://localhost:8000 |

# Known issues

- After adding a podcast refresh the browser. There is currently no websocket support available so after performing a manual action like downloading a new podcast episode you need to refresh the browser. 

# UI

## Audio Player

The podcast listening tool contains an advanced audio player that can be used to listen to your podcasts,skip episodes, turn the volumes as high as 300% or skip around in the current episode.
![Audio Player](https://raw.githubusercontent.com/samuel19982/podgrabv2/main/docs/advanced_audio_player.png)

# Continue right where you stopped

The tool will automatically save your progress in the current episode and will resume from where you left off even if you close the browser. 
You can continue listening on all devices by just hitting play on any episode on your home screen.

![Continue listening to episodes](https://raw.githubusercontent.com/samuel19982/podgrabv2/main/docs/continue_listening.png)

## Search for podcasts
You can search for podcast episodes by hitting CTRL+F and typing any word that might appear in the description or title of the podcast episode you want to listen to.

## Internationalization
Podgrab is currently available in English and German. If you want to add a new language you can do so by adding a new file to the `i18n` folder and adding the translations to the file.

# Roadmap

- [x] Add podcasts via Itunes API
- [x] Check for new episodes.
- [x] Download episodes.
- [x] Play episodes.
- [x] Force refresh download of podcast episodes.
- [x] Force refresh of podcast episodes.
- [x] Resume podcasts even if browser is closed. 
- [] Delete/Unsubscribe podcasts.
- [x] Add websocket support for new podcasts