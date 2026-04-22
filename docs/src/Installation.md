# Installation

## Installation with Docker

## Installation with Docker (SQLite)

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
      - DATABASE_URL=sqlite:///app/db/podcast.db

volumes:
    podfetch-podcasts:
    podfetch-db:
```



| Variable         | Description                                          | Default                  |
|------------------|------------------------------------------------------|--------------------------|
| POLLING_INTERVAL | Interval in minutes to check for new episodes        | 300                      |
| PORT             | The port PodFetch listens on                         | 8000                     |
| SUB_DIRECTORY    | Sub-path when hosting behind a reverse proxy (e.g. `/podfetch`) | _(none)_      |
| DATABASE_URL     | URL of the database                                  | sqlite://./db/podcast.db |
| PODFETCH_FOLDER  | Directory (inside the container) where podcast files are stored | podcasts     |

It is important to change `UID` and `GID` to your user id and group id so that the files are owned by you and not by root.
Docker will create the volumes by default as root and podfetch will not be able to write to them.

### Storing podcasts on a different drive

In most cases you don't need `PODFETCH_FOLDER`. To put podcast files on a different drive (e.g. a separate HDD while the rest of the container stays on an SSD), just bind-mount that drive to the default `/app/podcasts`:

```yaml
volumes:
  - /mnt/hdd/podcasts:/app/podcasts
  - podfetch-db:/app/db
```

If you do want to change the in-container path, set `PODFETCH_FOLDER` and mount the drive at that path. The directory (and any missing parents) will be created on startup.

## Installation with Docker (Postgres)

To use postgres you need to set the following environment variables:

```yaml
- DATABASE_URL=postgres://postgres:postgres@postgres:5432/podfetch
```


## Installation without Docker

### Requirements
- Download the latest release from the [release page](https://github.com/SamTV12345/PodFetch/releases)
- Create a shell script that sets the above environment variables and starts the podfetch binary
- Make the shell script executable
- Run the shell script

### Terraform

For terraform have a look at the setup directory.
There you will find everything needed to start with your infrastructure as code.