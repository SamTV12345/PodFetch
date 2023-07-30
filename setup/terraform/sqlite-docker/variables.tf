variable "db_dir" {
    default = "/var/podfetch/db"
}

variable "podcast_dir" {
  default = "/var/podfetch/podcasts"
}

variable "server_url" {
    default = "http://podfetch.example.com"
}

variable "traefik-http-port" {
    default = "80"
}

variable "traefik-https-port" {
  default = "443"
}

variable "traefik_toml_location" {
  description = "The location of the traefik.toml file"
  default     = "/etc/traefik/traefik.toml"
}

variable "traefik_acme_location" {
  description = "The location of the acme.json file"
  default     = "/etc/traefik/acme.json"
}


variable "traefik_access_log_location" {
  description = "The location of the access.log file"
  default     = "/var/log/traefik/access.log"
}

variable "traefik_dynamic_conf_location" {
  description = "The location of the dynamic configuration files"
  default     = "/etc/traefik/dynamic_conf"
}