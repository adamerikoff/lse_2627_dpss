APP_DIR := app
BIN_DIR := $(APP_DIR)/target/debug

.PHONY: db-up db-down db-reset db-logs run

db-up:
	docker-compose up -d

db-down:
	docker-compose down

# Drops the volume, so initdb/*.sql runs again.
db-reset:
	docker-compose down -v
	docker-compose up -d

db-logs:
	docker-compose logs -f db

run:
	cargo build --manifest-path $(APP_DIR)/Cargo.toml
	cp $(APP_DIR)/config.toml $(BIN_DIR)/config.toml
	$(BIN_DIR)/app
