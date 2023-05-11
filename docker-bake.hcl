# docker-bake.hcl
group "default" {
  targets = ["podfetch-amd64","podfetch-arm64","podfetch-armv7"]
}

target "podfetch-amd64" {
  args ={
    FILE="x86_64-unknown-linux-gnu"
  }
  dockerfile = "Dockerfile_cross"
  platforms = ["linux/amd64"]
  tags= "samuel19982/podfetch:dev"
}

target "podfetch-arm64" {
  args ={
    FILE="aarch64-unknown-linux-gnu"
  }
  dockerfile = "Dockerfile_cross"
  platforms = ["linux/arm64"]
  tags= "samuel19982/podfetch:dev"
}

target "podfetch-armv7" {
  args ={
    FILE="armv7-unknown-linux-gnueabih"
  }
  dockerfile = "Dockerfile_cross"
  platforms = ["linux/arm/v7"]
  tags= "samuel19982/podfetch:dev"
}