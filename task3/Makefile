APP_DIR := app
COMPOSE := docker-compose
PG := lse-dpss-task3-2026-postgres

.PHONY: up down reset nuke logs psql run runs app apps

# ---------------------------------------------------------------------------
#  Стек
# ---------------------------------------------------------------------------

# Собрать образ клиента и поднять postgres + клиент. Скрипты из app/sql
# выполняются при первой инициализации тома.
up:
	$(COMPOSE) up -d --build

down:
	$(COMPOSE) down

# Удалить том, чтобы 01..03 выполнились заново на чистой базе.
reset:
	$(COMPOSE) down -v
	$(COMPOSE) up -d --build

nuke:
	$(COMPOSE) down -v --remove-orphans

# Лог сервера.
logs:
	$(COMPOSE) logs -f db

# Консоль psql как суперпользователь (для проверки, не для клиента).
psql:
	$(COMPOSE) exec db psql -U postgres -d neobank

# ---------------------------------------------------------------------------
#  Клиент. Логин и пароль спрашиваются при запуске.
#
#  В контейнере (в одной сети с СУБД, п. 6.9), через exec с TTY:
#      make app      уязвимая версия
#      make apps     защищённая версия
#
#  На хосте (против 127.0.0.1:55434), для удобства:
#      make run      уязвимая версия
#      make runs     защищённая версия
# ---------------------------------------------------------------------------

app:
	$(COMPOSE) exec app /app/app

apps:
	$(COMPOSE) exec app /app/app --secure

run:
	cargo run --manifest-path $(APP_DIR)/Cargo.toml

runs:
	cargo run --manifest-path $(APP_DIR)/Cargo.toml -- --secure
