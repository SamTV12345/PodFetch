variable "public_port" {
    description = "The public port to access the application"
    default     = 80
}

variable "public_port_https" {
    description = "The public port to access the application"
    default     = 443
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