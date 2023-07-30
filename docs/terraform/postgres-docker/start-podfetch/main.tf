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
  image   = "samuel19982/podfetch:postgres"
  restart = "always"
  labels {
    label = "traefik.enable"
    value = "true"
  }

  labels{
    label = "traefik.http.routers.podfetch.rule"
    value = "Host(`${replace(var.server_url,"/(https?://)|(/)/","")}`)"
  }

  networks_advanced {
    name = "postgres-traefik-proxy"
  }

  networks_advanced {
    name = "podfetch-internal"
  }

  env = [
    "SERVER_URL=${var.server_url}",
    "DATABASE_URL=postgres://${var.db_user}:${var.db_password}@podfetch-db:5432/${var.db_name}",
  ]

  ports {
    internal = 8000
    external = 8000
  }

  volumes {
    container_path = "/app/podcasts"
    host_path      = var.podcast_dir
  }
}