# syntax=docker/dockerfile:1

# ============================================================================
#  build stage  —  compile the release binary
# ============================================================================
FROM rust:1-slim AS build
WORKDIR /src
COPY . ./
RUN cargo build --release

# ============================================================================
#  runtime stage  —  just the binary + its config
# ============================================================================
FROM debian:stable-slim
WORKDIR /app
COPY --from=build /src/target/release/app /app/app
COPY config.toml /app/config.toml

# `docker run` this image runs the looping client. As a compose service it is
# overridden to idle (see docker-compose.yml) so the client can be launched
# on demand with an attached TTY via `make app`.
CMD ["/app/app"]
