# Podgrab V2

Podgrab is a self-hosted podcast manager. 
It is a web app that lets you download podcasts and listen to them online.
It is written in Rust and uses React for the frontend.


# Getting Started

## Docker

### Docker Compose

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
