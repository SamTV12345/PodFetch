variable "server_url" {
    default = "http://podfetch.example.com"
    description = "The URL of the podfetch server"
}

variable "podcast_dir" {
    default = "/var/podfetch/podcasts"
    description = "The directory where podcasts are stored"
}


variable "db_dir" {
    default = "/var/podfetch/db"
    description = "The directory where the podfetch database is stored"
}

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