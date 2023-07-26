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


## Usage in docker

```bash
docker ps #This will help you obtain the container's id and name
docker exec -it <container id or name> /app/podfetch <your-command> # Will execute your desired command in the container
```
