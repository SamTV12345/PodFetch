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

# Roadmap

- [x] Add podcasts via Itunes API
- [x] Check for new episodes.
- [x] Download episodes.
- [x] Play episodes.
- [x] Force refresh download of podcast episodes.
- [x] Force refresh of podcast episodes.
- [x] Resume podcasts even if browser is closed. 
- [] Delete/Unsubscribe podcasts.
- [] Add websocket support for new podcasts