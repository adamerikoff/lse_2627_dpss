APP_DIR := app
BIN_DIR := $(APP_DIR)/target/debug
COMPOSE := docker-compose

.PHONY: up down reset logs nuke app run

# Build images and start the whole stack (postgres + app). The app container
# idles; launch the binary with `make app`.
up:
	$(COMPOSE) up -d --build

# Stop and remove the stack (the named data volume is kept).
down:
	$(COMPOSE) down

# Drop the volume so the database is initialised from scratch, then restart.
reset:
	$(COMPOSE) down -v
	$(COMPOSE) up -d --build

# Follow the postgres server log.
logs:
	$(COMPOSE) logs -f db

# Complete reset: containers, volume, locally-built images, orphans.
nuke:
	$(COMPOSE) down -v --rmi local --remove-orphans

# Run the binary INSIDE its container (interactive TTY, reaches db:5432).
app:
	$(COMPOSE) exec app /app/app

# Run the binary on the HOST, against the published port.
run:
	cargo build --manifest-path $(APP_DIR)/Cargo.toml
	$(BIN_DIR)/app
