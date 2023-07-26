# Podfetch

[![dependency status](https://deps.rs/repo/github/SamTV12345/PodFetch/status.svg)](https://deps.rs/repo/github/SamTV12345/PodFetch)

Podfetch is a self-hosted podcast manager.
It is a web app that lets you download podcasts and listen to them online.
It is written in Rust and uses React for the frontend.
It also contains a GPodder integration so you can continue using your current podcast app.

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
# File just needs to be there
touch static/index.html
cargo.exe run --color=always --package podfetch --bin podfetch
cd ui
<npm/yarn/pnpm> install
<npm/yarn/pnpm> run dev
```

If you want to run other databases you need to install the corresponding diesel cli. For example for postgres you need to install `diesel_cli --no-default-features --features postgres` and run the same command for running it with cargo.

## UI Development

I would love to have a UX expert to help me with the UI. If you are interested in helping me out, please contact me via GitHub issue with designs/implemented React pages.

# Getting Started

## Docker

### Docker-Compose Examples

#### Docker-Compose

##### Advantages over Postgres
- Easier to setup
- Easier to use

=> No concurrency. So please don't try to download to podcasts at the same time.


### Sqlite

```yaml
version: '3'
services:
  podfetch:
    image: samuel19982/podfetch:latest
    user: ${UID:-1000}:${GID:-1000}
    ports:
      - "80:8000"
    volumes:
      - podfetch-podcasts:/app/podcasts
      - podfetch-db:/app/db
    environment:
      - POLLING_INTERVAL=60
      - SERVER_URL=http://<your-ip>:<your-port>

volumes:
  podfetch-podcasts:
  podfetch-db:
```

### Postgres

#### Advantages over SQLite

- Better performance
- Better concurrency
- Better stability
- Better scalability

#### Docker Compose

```yaml
version: '3'
services:
  podfetch:
    image: samuel19982/podfetch:postgres
    user: ${UID:-1000}:${GID:-1000}
    ports:
      - "80:8000"
    volumes:
      - ./podcasts:/app/podcasts
    environment:
      - POLLING_INTERVAL=300
      - SERVER_URL=http://localhost:80 # Adjust to your server url
      - DATABASE_URL=postgresql://postgres:changeme@postgres/podfetch
      - DB_CONNECTIONS=10 # optional
  postgres:
    image: postgres
    environment:
      POSTGRES_USER: ${POSTGRES_USER:-postgres}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-changeme}
      PGDATA: /data/postgres
      POSTGRES_DB: ${POSTGRES_DB:-podfetch}
    volumes:
      - postgres:/data/postgres
    restart: unless-stopped

volumes:
  postgres:
```

# Auth

Several Auth methods are described here: [AUTH.md](docs/AUTH.md)

# Hosting

Hosting options are described here: [HOSTING.md](docs/HOSTING.md)

# CLI Usage

The CLI usage is described here: [CLI.md](docs/CLI.md)

# User Creation

You can create an admin, user, or uploader either through [CLI](docs/CLI.md) or via invites.

To generate an invite, log into Podfetch → Top Right Icon → User Administration → Invites

# Environment Variables

| Variable         | Description                                   | Default                  |
|------------------|-----------------------------------------------|--------------------------|
| POLLING_INTERVAL | Interval in minutes to check for new episodes | 300                      |
| SERVER_URL       | URL of the server                             | http://localhost:8000    |
| DATABASE_URL     | URL of the database                           | sqlite://./db/podcast.db |


# UI

[UI Documentation](docs/UIWalkthrough.md)

## Internationalization
Podfetch is currently available in English and German. If you want to add a new language you can do so by adding a new file to the `i18n` folder and adding the translations to the file.

# RSS Feed

Podfetch offers an own feed to download podcast episodes. You can add the url <SERVER_URL>/rss to your favorite podcast app like gPodder to download and play episodes.

# Podcast Index

It is also possible to retrieve/add podcasts from [Podcast Index](https://podcastindex.org/).
To configure it you need to create an account on that website. After creating an account an email is sent to you with the required credentials.


| Variable            | Description                           | Default |
|---------------------|---------------------------------------|---------|
| PODINDEX_API_KEY    | the api key sent to you via mail      | %       |
| PODINDEX_API_SECRET | the api secret also found in the mail | %       |

* % means an empty string is configured as default

After successful setup you should see on the settings page a green checkmark next to the Podindex config section.

# GPodder API

Podfetch supports the [GPodder API](https://gpoddernet.readthedocs.io/en/latest/api/index.html).

The following environment variable must be set to `true` to enable it:
| Variable            | Description                           | Default |
|---------------------|---------------------------------------|---------|
| GPODDER_INTEGRATION_ENABLED    | Activates the GPodder integration via your `SERVER_URL` | false|

You will also need to set up [`BASIC_AUTH` or `OIDC_AUTH`](docs/AUTH.md) and [create a user](#user-creation).

You can use your new user account to log into podcast apps that supports the GPodder API by using your `SERVER_URL` and login information.

# Roadmap

- [x] Add podcasts via Itunes api
- [x] Check for new episodes.
- [x] Download episodes.
- [x] Play episodes.
- [x] Force refresh download of podcast episodes.
- [x] Force refresh of podcast episodes.
- [x] Resume podcasts even if browser is closed.
- [x] Add websocket support for new podcasts.
- [x] Add detailed audio player.
- [x] Star podcasts.
- [x] Unsubscribe podcasts.
- [x] Add retrieving podcasts from Podcastindex.org.
- [x] Basic Auth.
- [x] Import from OPML file.
- [x] Telegram Bot api to get alerted when new episodes are downloaded.
- [ ] Like episodes.
- [ ] Delete podcasts.

