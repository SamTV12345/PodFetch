variable "path-to-samples" {
    default = "./samples"
}

resource "local_file" "traefik_toml" {
  source      = "${var.path-to-samples}/traefik.toml"
  filename = var.traefik_toml_location
}
resource "local_file" "traefik_access_log" {
  content = "<<EOF"
  filename = var.traefik_access_log_location
}

resource "local_file" "traefik-dynamic-conf" {
  source      = "${var.path-to-samples}/dynamic.toml"
  filename = var.traefik_dynamic_conf_location
}