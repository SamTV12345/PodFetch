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