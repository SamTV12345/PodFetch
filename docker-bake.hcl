group "default" {
  targets = ["podfetch"]
}

target "podfetch" {
  dockerfile = "Dockerfile_cross"
  tags= ["samuel19982/podfetch:dev"]
  platforms = ["linux/amd64", "linux/arm64", "linux/arm/v7"],
  args = {
    CACHEBUST = "1"
  }
}