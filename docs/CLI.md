# CLI usage

The CLI can be used to create and manage users, and also to refresh and list podcasts.

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


## Usage in docker

```bash
docker ps #This will help obtain your PodFetch container's name
docker exec -it <your-docker-id/name> /app/podfetch <your-command> # Will execute your desired command in the container
```
