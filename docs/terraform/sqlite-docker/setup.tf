terraform {
  required_providers {
    docker = {
      source  = "kreuzwerker/docker"
      version = "3.0.2"
    }
  }
}

provider "docker" {
  host  = "unix:///var/run/docker.sock"
}


module "prepare-traefik" {
  source = "./prepare-traefik"
  traefik_access_log_location   = var.traefik_access_log_location
  traefik_acme_location         = var.traefik_acme_location
  traefik_dynamic_conf_location = var.traefik_dynamic_conf_location
  traefik_toml_location         = var.traefik_toml_location
}

module "deploy-traefik-images" {
  source = "./start-traefik"
  depends_on = [module.prepare-traefik]
  providers = {
    docker = docker
  }
  public_port                   = var.traefik-http-port
  public_port_https             = var.traefik-https-port
  traefik_access_log_location   = var.traefik_access_log_location
  traefik_acme_location         = var.traefik_acme_location
  traefik_dynamic_conf_location = var.traefik_dynamic_conf_location
  traefik_toml_location         = var.traefik_toml_location
}

module "deploy-podfetch" {
  source = "./start-podfetch"

  depends_on = [module.deploy-traefik-images]
  db_dir     = var.db_dir
  podcast_dir = var.podcast_dir
  server_url = var.server_url
}