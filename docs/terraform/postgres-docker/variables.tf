variable "postgres_user" {
  default = "podfetch"
  description = "The postgres user for podfetch"
}


variable "postgres_password" {
  default = "podfetch"
  description = "The postgres password for podfetch"
}

variable "postgres_db" {
  default = "podfetch"
  description = "The postgres database for podfetch"
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