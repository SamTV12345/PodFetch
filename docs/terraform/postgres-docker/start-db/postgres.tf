terraform {
  required_providers {
    docker = {
      source = "kreuzwerker/docker"
      version = "3.0.2"
    }
  }
}
resource "docker_network" "podfetch-internal" {
  name = "podfetch-internal"
}
resource "docker_container" "postgres-db" {
  name    = "podfetch-db"
  image   = "postgres"
  restart = "always"

  networks_advanced {
    name = "podfetch-internal"
  }

  env = [
    "POSTGRES_USER=${var.db_user}",
    "POSTGRES_PASSWORD=${var.db_password}",
    "POSTGRES_DB=${var.db_name}"
  ]

  volumes {
    container_path = "/var/lib/postgresql/data"
    host_path      = var.db_dir
    read_only      = false
  }

  labels {
    label = "traefik.enable"
    value = "false"
  }


}