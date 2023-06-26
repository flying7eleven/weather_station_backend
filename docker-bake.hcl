target "docker-metadata-action" {}
target "build" {
  inherits   = ["docker-metadata-action"]
  context    = "./"
  dockerfile = "Dockerfile"
  platforms  = [
    # "linux/amd64", # x86_64 processors
    "linux/arm64", # ARMv8, also called AArch64 (Raspberry Pi 4+, Apple Mx processors)
    # "linux/arm/v7", # ARMv7 (Raspberry Pi 3 compatibility mode)
  ]
}
