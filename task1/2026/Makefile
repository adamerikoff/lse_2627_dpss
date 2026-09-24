APP_DIR := app
BIN_DIR := $(APP_DIR)/target/debug
COMPOSE := docker-compose

.PHONY: up down reset logs nuke app run

# Build images and start the whole stack (postgres + app). The app container
# idles; launch the interactive client with `make app`.
up:
	$(COMPOSE) up -d --build

# Stop and remove the stack (the named data volume is kept).
down:
	$(COMPOSE) down

# Drop the volume so initdb/*.sql runs again, then start fresh.
reset:
	$(COMPOSE) down -v
	$(COMPOSE) up -d --build

# Follow the postgres server log (the auth method is visible here).
logs:
	$(COMPOSE) logs -f db

# Complete reset: remove containers, the volume, locally-built images, and any
# orphaned containers left from earlier compose files.
nuke:
	$(COMPOSE) down -v --rmi local --remove-orphans

# Run the looping client INSIDE its container (interactive TTY, reaches db:5432).
app:
	$(COMPOSE) exec app /app/app

# Run the looping client on the HOST, against the published 127.0.0.1:55432.
run:
	cargo build --manifest-path $(APP_DIR)/Cargo.toml
	cp $(APP_DIR)/config.toml $(BIN_DIR)/config.toml
	$(BIN_DIR)/app
