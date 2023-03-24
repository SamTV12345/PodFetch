# Podfetch

Podfetch is a self-hosted podcast manager. 
It is a web app that lets you download podcasts and listen to them online.
It is written in Rust and uses React for the frontend.

Every time a new commit is pushed to the main branch, a new docker image is built and pushed to docker hub. So it is best to use something like [watchtower](https://github.com/containrrr/watchtower) to automatically update the docker image.

# Contributing

## Building the project

### Prerequisites
- Rust
- Cargo
- Node
- npm/yarn/pnpm

### Building the app
```bash
cargo.exe run --color=always --package podgrabv2 --bin podgrabv2
cd ui
<npm/yarn/pnpm> install
<npm/yarn/pnpm> run dev
```

## UI Development

I would love to have a UX expert to help me with the UI. If you are interested in helping me out, please contact me via GitHub issue with designs/implemented React pages.

# Getting Started

## Docker

### Docker-Compose

```yaml
version: '3'
services:
  podfetch:
    image: samuel19982/podgrabv2:latest
    ports:
      - "80:8000"
    volumes:
      - podgrab-podcasts:/app/podcasts
      - podgrab-db:/app/db/podcast.db
    environment:
      - POLLING_INTERVAL=60
      - SERVER_URL=http://<your-ip>:<your-port>

volumes:
  podgrab-podcasts:
  podgrab-db:
```

# Environment Variables

| Variable         | Description                                   | Default                  |
|------------------|-----------------------------------------------|--------------------------|
| POLLING_INTERVAL | Interval in minutes to check for new episodes | 60                       |
| SERVER_URL       | URL of the server                             | http://localhost:8000    |
| DATABASE_URL     | URL of the database                           | sqlite::/./db/podcast.db |

# Known issues


# UI

## Audio Player
The podcast listening tool contains an advanced audio player that can be used to listen to your podcasts,skip episodes, turn the volumes as high as 300% or skip around in the current episode.
![Audio Player](https://raw.githubusercontent.com/SamTV12345/podgrabv2/main/docs/advanced_audio_player.png)

# Continue right where you stopped

The tool will automatically save your progress in the current episode and will resume from where you left off even if you close the browser. 
You can continue listening on all devices by just hitting play on any episode on your home screen.

![Continue listening to episodes](https://raw.githubusercontent.com/SamTV12345/podgrabv2/main/docs/continue_listening.png)

## Search for podcasts
You can search for podcast episodes by hitting CTRL+F and typing any word that might appear in the description or title of the podcast episode you want to listen to.
![Audio Player](https://raw.githubusercontent.com/SamTV12345/podgrabv2/main/docs/search.png)

## Internationalization
Podfetch is currently available in English and German. If you want to add a new language you can do so by adding a new file to the `i18n` folder and adding the translations to the file.

# Roadmap

- [x] Add podcasts via Itunes API
- [x] Check for new episodes.
- [x] Download episodes.
- [x] Play episodes.
- [x] Force refresh download of podcast episodes.
- [x] Force refresh of podcast episodes.
- [x] Resume podcasts even if browser is closed.
- [x] Add websocket support for new podcasts.
- [ ] Add detailed audio player.
- [ ] Delete/Unsubscribe podcasts.
- [ ] Star podcasts.
- [ ] Like episodes.