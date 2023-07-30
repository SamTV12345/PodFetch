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


module "deploy-traefik-images" {
  source = "./start-traefik"
  providers = {
    docker = docker
  }
}

module "deploy-podfetch" {
  source = "./start-podfetch"

  depends_on = [module.deploy-traefik-images]
}