# CLI usage

The CLI can be used to update, remove, list registered users in PodFetch. You can get help anytime by typing --help/help

# Usage

# Get general help

```bash
podfetch --help
```

# Get help for a specific command

```bash
podfetch <command> --help
```

e.g. 

```bash
podfetch users --help
```


# Usage in docker

```bash
docker ps #This will get you the id of the
docker exec -it <your-docker-id/name> /app/podfetch <your-command> # Will execute your desired command in the container
```
