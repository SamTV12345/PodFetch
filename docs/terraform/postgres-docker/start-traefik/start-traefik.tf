terraform {
  required_providers {
    docker = {
      source = "kreuzwerker/docker"
      version = "3.0.2"
    }
  }
}





resource "docker_container" "traefik" {
  name = "traefik_proxy"
  image = "traefik"
  restart = "always"
  ports {
    internal = 80
    external = var.public_port
  }

  ports {
    internal = 443
    external = var.public_port_https
  }

  networks_advanced {
    name = "traefik-proxy"
  }

  volumes {
    container_path = "/var/run/docker.sock"
    host_path = "/var/run/docker.sock"
    read_only = false
  }

  volumes {
    container_path = "/etc/traefik/traefik.toml"
    host_path = var.traefik_toml_location
    read_only = true
  }

  volumes {
    container_path = "/etc/traefik/acme.json"
    host_path = var.traefik_acme_location
    read_only = false
  }

  volumes {
        container_path = "/etc/traefik/dynamic.toml"
        host_path = var.traefik_dynamic_conf_location
        read_only = true
    }

  volumes {
    container_path = "/var/log/traefik/access.log"
    host_path = var.traefik_access_log_location
    read_only = false
  }
}