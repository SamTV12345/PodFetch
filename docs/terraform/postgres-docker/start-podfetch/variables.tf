variable "server_url" {
    default = "http://podfetch.example.com"
    description = "The URL of the podfetch server"
}

variable "podcast_dir" {
    description = "The directory where podcasts are stored"
}


variable "db_user" {
    description = "The database user"
}

variable "db_password" {
    description = "The database password"
}

variable "db_name" {
    description = "The database name"
}
