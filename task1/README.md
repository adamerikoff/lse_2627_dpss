# Задание №1 — PostgreSQL 18 + клиент на Rust

Один клиент, два способа собрать **строку подключения**:

- **уязвимый** (режим `i`) — conninfo libpq собирается конкатенацией;
- **защищённый** (режим `s`) — значения кладутся в поля `postgres::Config`.

Сценарии ниже показывают инъекцию в строку подключения в уязвимом режиме и ту
же попытку в защищённом.

## Запуск

```sh
make up        # собрать образ, поднять postgres (127.0.0.1:55432) + клиент
make reset     # пересоздать БД с нуля (publicdb + secretdb заново)

make app       # клиент в контейнере (в одной сети с СУБД)
make run       # клиент на хосте (против 127.0.0.1:55432)
make logs      # лог сервера (виден метод аутентификации)
make down
```

Клиент в цикле спрашивает режим (`s`/`i`), затем логин и пароль; после отчёта
спрашивает снова — можно перебирать нагрузки подряд. Выход — `Ctrl-D`.

## Учётные данные

Корректные: **`usr` / `pass`**. Роль `usr` имеет `SELECT` на таблицу `data`
в **обеих** базах (`publicdb` и `secretdb`), но приложение штатно работает
только с `publicdb`.

Ключ демонстрации: `report()` выполняет **неквалифицированный** запрос
`SELECT * FROM data` — без имени базы. Какую таблицу он прочитает, решает строка
подключения. «Стеной» вокруг `secretdb` служит только намерение приложения, а не
права доступа.

---

## Сценарий 1. Утечка секретной базы

**Уязвимый (`i`).** Логин `usr`, пароль:

```
pass dbname=secretdb
```

→ Отчёт показывает `database = secretdb`, а блок `TABLE data` — секретные строки
(`API_KEY=…`, `db_superuser_password=…`). Пароль склеился в conninfo как
`password=pass dbname=secretdb`; libpq разобрал это как два параметра, и
`dbname=secretdb` перекрыл штатный `dbname=publicdb` (побеждает последнее
вхождение). Запрос `SELECT * FROM data` не менялся — сменилась база.

**Защищённый (`s`).** Тот же ввод.

→ Ошибка аутентификации. Строка `pass dbname=secretdb` целиком стала
литеральным паролем — он не совпал с `pass`. Утечки нет.

---

## Сценарий 2. Подмена параметров сессии

**Уязвимый (`i`).** Логин:

```
usr application_name=INJECTED
```

пароль `pass`.

→ В отчёте `application_name = INJECTED`. Тем же приёмом инъектируются и другие
параметры, например пароль `pass options=-csearch_path=information_schema`
подменяет `search_path` бэкенда.

**Защищённый (`s`).** Тот же ввод → логин целиком стал именем пользователя,
такой роли нет → отказ аутентификации.

---

## Проверка на уровне SQL (в обход приложения)

Показывает, что `usr` действительно авторизован в обеих базах — защищал только
выбор базы в приложении, а не `GRANT`:

```sh
docker exec -e PGPASSWORD=pass lse-dpss-task1-2026-postgres \
  psql -U usr -d publicdb -c 'SELECT id, label FROM data ORDER BY id'
docker exec -e PGPASSWORD=pass lse-dpss-task1-2026-postgres \
  psql -U usr -d secretdb -c 'SELECT id, label FROM data ORDER BY id'
```

## Справочник нагрузок

| Логин | Пароль | Уязвимый (`i`) | Защищённый (`s`) |
|---|---|---|---|
| `usr` | `pass` | штатно, `publicdb` | штатно, `publicdb` |
| `usr` | `pass dbname=secretdb` | **утечка** `secretdb.data` | отказ аутентификации |
| `usr` | `pass dbname=postgres` | подключение к `postgres` | отказ |
| `usr application_name=INJECTED` | `pass` | `application_name=INJECTED` | отказ |
| `usr` | `pass user=postgres` | попытка войти суперпользователем | отказ |
| `usr` | `pass options=-csearch_path=information_schema` | подмена `search_path` | отказ |

---

## В чём разница версий

`insecure.rs` склеивает conninfo:

```
host=127.0.0.1 port=55432 dbname=publicdb application_name=task1-app user=<USER> password=<PASS>
```

Драйвер разбирает строку обратно по пробелам и `=`, и при повторе ключа
побеждает последнее значение — поэтому всё после пробела становится опцией.

`secure.rs` присваивает каждое значение отдельному полю `postgres::Config`. Поля
уходят в StartupMessage протокола, где параметры разделены нулевыми байтами, и
пробел/`=` внутри значения остаются частью значения, а не разделителем.

## Устройство

```
db/initdb/     01-roles (роль usr), 02-schema (publicdb+secretdb, таблицы data), 03-seed
app/src/
  main.rs      цикл: режим, логин, пароль, отчёт
  mode.rs      разбор s/i
  prompt.rs    ввод, пароль из /dev/tty
  insecure.rs  conninfo конкатенацией
  secure.rs    поля postgres::Config
  report.rs    неквалифицированный SELECT * FROM data + состояние сессии
```

Разница версий — только в `insecure.rs` / `secure.rs`; остальное общее.
