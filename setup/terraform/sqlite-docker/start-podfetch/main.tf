terraform {
  required_providers {
    docker = {
      source = "kreuzwerker/docker"
      version = "3.0.2"
    }
  }
}


resource "docker_container" "podfetch" {
  name    = "podfetch"
  image   = "samuel19982/podfetch"
  restart = "always"
  labels {
    label = "traefik.enable"
    value = "true"
  }

  labels{
    label = "traefik.http.routers.podfetch.rule"
    value = "Host(`${var.hostname}`)"
  }

  networks_advanced {
    name = "sqlite-traefik-proxy"
  }

  env = []

  volumes {
    container_path = "/app/podcasts"
    host_path      = var.podcast_dir
  }

  volumes {
    container_path = "/app/db"
    host_path      = var.db_dir
  }
}