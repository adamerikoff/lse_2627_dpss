# syntax=docker/dockerfile:1

# ============================================================================
#  build stage  —  compile the release binary
# ============================================================================
FROM rust:1-slim AS build
WORKDIR /src
COPY . ./
RUN cargo build --release

# ============================================================================
#  runtime stage  —  just the binary
# ============================================================================
FROM debian:stable-slim
WORKDIR /app
COPY --from=build /src/target/release/app /app/app

CMD ["/app/app"]
