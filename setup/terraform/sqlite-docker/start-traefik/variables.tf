variable "public_port" {
    description = "The public port to access the application"
}

variable "public_port_https" {
    description = "The public port to access the application"
}


variable "traefik_toml_location" {
  description = "The location of the traefik.toml file"
}

variable "traefik_acme_location" {
    description = "The location of the acme.json file"
}


variable "traefik_access_log_location" {
    description = "The location of the access.log file"
}

variable "traefik_dynamic_conf_location" {
    description = "The location of the dynamic configuration files"
}