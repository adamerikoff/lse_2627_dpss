# Задание №1 — PostgreSQL 18 + клиент на Rust

Одно приложение, два способа собрать подключение.

## Запуск

```sh
make db-up      # postgres на 127.0.0.1:55432
make run        # сборка и запуск
make db-logs    # лог сервера (виден метод аутентификации)
make db-reset   # удалить том и заново выполнить db/initdb/*.sql
make db-down
```

Приложение спрашивает режим (`s` / `i`), затем логин и пароль. Корректные
учётные данные: `usr` / `pass`. Ввод пароля требует настоящего терминала —
он читается напрямую из `/dev/tty`.

## В чём разница

`insecure.rs` склеивает строку подключения libpq:

```
host=127.0.0.1 port=55432 dbname=publicdb application_name=task1-app user=<USER> password=<PASS>
```

Драйвер разбирает эту строку обратно, разделяя её по пробелам и `=`, причём при
повторении ключа побеждает **последнее** вхождение. Поэтому всё, что введено
после пробела, становится полноценной опцией подключения.

`secure.rs` присваивает каждое значение отдельному полю `postgres::Config`. Эти
поля попадают прямо в StartupMessage протокола PostgreSQL, где параметры
разделяются нулевыми байтами, — а нулевой байт внутри значения кодировщик
принять отказывается.

## Две базы и таблица `data`

`db/initdb/*.sql` создаёт **две** базы, и в каждой — таблицу с одинаковым именем
`data`:

- `publicdb` — то, что приложение **должно** показывать (безобидные строки);
- `secretdb` — то, куда приложение **никогда** не обращается (секреты).

Роль `usr` имеет права `SELECT` на **обе** таблицы. `report()` выполняет
**неквалифицированный** запрос `SELECT * FROM data` — без имени базы и схемы,
поэтому какую именно таблицу он прочитает, решает строка подключения. В штатном
режиме это `publicdb.data`; если инъекция подменит `dbname=secretdb`, тот же
самый запрос вернёт секреты. Единственное, что «защищало» `secretdb`, — это
намерение приложения остаться в `publicdb`, а не права доступа: инъекция
обходит намерение, а не `GRANT`.

### Расширенная таблица полезных нагрузок (Payloads)


| Логин (`user`) | Пароль (`password`) | Поведение в `insecure` | Поведение в `secure` | Влияние на вывод `report()` |
| --- | --- | --- | --- | --- |
| `usr` | `pass` | Подключение к `publicdb` | Подключение к `publicdb` | Штатное состояние сессии |
| `usr` | `pass dbname=postgres` | Подключение к **`postgres`** | Аутентификация не проходит | `database = postgres` |
| `usr` | `pass dbname=secretdb` | **Утечка данных**: `SELECT * FROM data` читает `secretdb.data` | Аутентификация не проходит | `database = secretdb` + дамп секретных строк |
| `usr application_name=INJECTED` | `pass` | Имя приложения меняется на **`INJECTED`** | Аутентификация не проходит | `application_name = INJECTED` |
| `usr` | `pass user=postgres` | Попытка подключения от имени суперпользователя `postgres` | Аутентификация не проходит | `user = postgres`<br>`session_user = postgres` |
| `usr` | `pass connect_timeout=1` | Принудительное ограничение тайм-аута до 1 секунды | Аутентификация не проходит | Ошибка таймаута при задержках сети |
| `usr` | `pass options=-csearch_path=information_schema` | Инъекция аргументов командной строки бэкенда | Аутентификация не проходит | `search_path = information_schema, ...` |

### Как увидеть утечку (команды)

Свежий том, чтобы `initdb` создал `publicdb` и `secretdb` заново:

```sh
make db-reset
```

**1. Базовая линия — штатный запуск.** Приложение остаётся в `publicdb`:

```text
make run
mode [s]ecure / [i]nsecure: i
username: usr
password: pass
```

`report()` покажет `database = publicdb`, а блок `TABLE data` — три публичные
строки.

**2. Инъекция — утечка секретов (`insecure`).** Всё после `pass ` становится
опциями подключения, и `dbname=secretdb` перекрывает `dbname=publicdb`:

```text
make run
mode [s]ecure / [i]nsecure: i
username: usr
password: pass dbname=secretdb
```

Теперь `report()` покажет `database = secretdb`, а блок `TABLE data` выведет
секретные строки (`API_KEY=...`, `db_superuser_password=...`, …). Код запроса
`SELECT * FROM data` не менялся — сменилась только база.

**3. Тот же ввод в `secure`-режиме — отказа.** `pass dbname=secretdb` целиком
становится литеральным паролем, он не совпадает с `pass`, соединение падает с
ошибкой аутентификации, утечки нет:

```text
make run
mode [s]ecure / [i]nsecure: s
username: usr
password: pass dbname=secretdb
```

**4. Проверка на уровне SQL, в обход приложения (без TTY).** Показывает, что
`usr` действительно авторизован в обеих базах — «стеной» служил только выбор
базы в приложении:

```sh
# публичные данные
docker exec -e PGPASSWORD=pass lse-dpss-task1-2026-postgres \
  psql -U usr -d publicdb -c 'SELECT id, label FROM data ORDER BY id'

# секреты
docker exec -e PGPASSWORD=pass lse-dpss-task1-2026-postgres \
  psql -U usr -d secretdb -c 'SELECT id, label FROM data ORDER BY id'
```