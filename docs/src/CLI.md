# CLI usage

The CLI can be used to manage users and to refresh & list subscribed podcasts.

You can get help anytime by typing `--help` or `help`.

# Usage

## Get general help

```bash
podfetch --help
```

## Get help for a specific command

```bash
podfetch <command> --help
```

e.g. 

```bash
podfetch users --help
podfetch podcasts --help
```

## Running as a Chromecast agent

PodFetch can also run in agent mode, where it does not start an HTTP
server but instead connects to a remote PodFetch instance over
websocket and forwards Chromecast control commands to devices on the
local LAN. See the [Chromecast](./Chromecast.md) chapter for the full
flow.

```bash
podfetch --agent \
  --remote https://podfetch.example.com \
  --api-key YOUR_USER_API_KEY \
  --agent-id home-lan
```


## Usage in docker

```bash
docker ps #This will help you obtain the container's id and name
docker exec -it <container id or name> /app/podfetch <your-command> # Will execute your desired command in the container
```