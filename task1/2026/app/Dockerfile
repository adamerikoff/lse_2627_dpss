# syntax=docker/dockerfile:1

# ============================================================================
#  build stage  —  compile the release binary
# ============================================================================
# Build context is this `app/` directory (see docker-compose.yml). rust:1-slim
# tracks the latest stable 1.x toolchain (>= 1.85), which supports edition 2024
# declared in Cargo.toml. Pin a version here if a reproducible toolchain matters.
FROM rust:1-slim AS build
WORKDIR /src
COPY . ./
RUN cargo build --release

# ============================================================================
#  runtime stage  —  just the binary + its config
# ============================================================================
# The binary links glibc (built on debian) and uses NoTls, so no OpenSSL/CA
# packages are needed. config.rs looks for config.toml next to the executable,
# hence both land in /app.
FROM debian:stable-slim
WORKDIR /app
COPY --from=build /src/target/release/app /app/app
COPY config.toml /app/config.toml

# `docker run` this image runs the looping client. As a compose service it is
# overridden to idle (see docker-compose.yml) so the client can be launched
# on demand with an attached TTY via `make app`.
CMD ["/app/app"]
