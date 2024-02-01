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
      - SERVER_URL=http://localhost:80
      - DATABASE_URL=sqlite:///app/db/podcast.db

volumes:
    podfetch-podcasts:
    podfetch-db:
```



| Variable         | Description                                   | Default                  |
|------------------|-----------------------------------------------|--------------------------|
| POLLING_INTERVAL | Interval in minutes to check for new episodes | 300                      |
| SERVER_URL       | URL of the server/the URL of the proxy        | http://localhost:8000    |
| DATABASE_URL     | URL of the database                           | sqlite://./db/podcast.db |

It is important to change `UID` and `GID` to your user id and group id so that the files are owned by you and not by root.
Docker will create the volumes by default as root and podfetch will not be able to write to them.

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