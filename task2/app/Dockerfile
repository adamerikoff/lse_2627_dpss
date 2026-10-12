# syntax=docker/dockerfile:1

# ============================================================================
#  сборка — пять бинарников из одного crate
# ============================================================================
FROM rust:1-slim AS build
WORKDIR /src
COPY . ./
RUN cargo build --release

# ============================================================================
#  исполнение — бинарники плюс файл конфигурации (п. 1.8)
# ============================================================================
FROM debian:stable-slim
WORKDIR /app

COPY --from=build /src/target/release/fails1 /app/fails1
COPY --from=build /src/target/release/fails2 /app/fails2
COPY --from=build /src/target/release/fails3 /app/fails3
COPY --from=build /src/target/release/fails4 /app/fails4
COPY --from=build /src/target/release/failsnone /app/failsnone

# config.toml ищется рядом с исполняемым файлом.
COPY config.toml /app/config.toml

# Образ по умолчанию запускает защищённый вариант; compose переопределяет
# command для каждого из пяти сервисов.
CMD ["/app/failsnone"]
