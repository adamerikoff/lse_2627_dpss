// Слой ОПЕРАЦИЙ над данными. Две реализации одного трейта, выбор по флагу
// --secure. Операции без пользовательского ввода одинаковы у обеих и берутся
// из общих помощников здесь; расходятся методы, принимающие значения и имена
// колонок (filter и далее).

use crate::db::insecure::Insecure;
use crate::db::secure::Secure;
use postgres::Client;
use postgres::Row;
use postgres::SimpleQueryMessage;

// Результат чтения: заголовки и строки, всё уже как текст для показа.
pub struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub trait Commands {
    // Роль сессии: bank_employee при обычном входе, bank_admin при эскалации.
    fn current_user(&self, client: &mut Client) -> Result<String, String>;

    // Список таблиц из каталога.
    fn list_tables(&self, client: &mut Client) -> Result<Vec<String>, String>;

    // 6.1.1: SELECT * без фильтра. Имя таблицы — из каталога, не свободный ввод.
    fn select_all(&self, client: &mut Client, table: &str) -> Result<Table, String>;

    // 6.1.2: SELECT * WHERE <колонка> = <значение>. Здесь расходятся версии:
    // имя колонки и значение приходят от пользователя.
    fn filter(
        &self,
        client: &mut Client,
        table: &str,
        column: &str,
        value: &str,
    ) -> Result<Table, String>;

    // 6.1.3: SELECT * WHERE <колонка> IN (<значения>). Переменное число
    // значений — группа однородных данных из п. 6.13.
    fn filter_in(
        &self,
        client: &mut Client,
        table: &str,
        column: &str,
        values: &[String],
    ) -> Result<Table, String>;

    // 6.2.1: UPDATE <таблица> SET <колонка> = <значение> WHERE <колонка> = <значение>.
    // Возвращает число изменённых строк.
    fn update(
        &self,
        client: &mut Client,
        table: &str,
        set_column: &str,
        set_value: &str,
        where_column: &str,
        where_value: &str,
    ) -> Result<u64, String>;

    // 6.2.2: то же, но WHERE <колонка> IN (<значения>) — несколько строк.
    fn update_in(
        &self,
        client: &mut Client,
        table: &str,
        set_column: &str,
        set_value: &str,
        where_column: &str,
        where_values: &[String],
    ) -> Result<u64, String>;

    // 6.3.1: INSERT одной строки. columns — из каталога (доверенные),
    // values — от пользователя. Пустые колонки сюда не попадают: их
    // значения БД подставляет по умолчанию.
    fn insert(
        &self,
        client: &mut Client,
        table: &str,
        columns: &[String],
        values: &[String],
    ) -> Result<u64, String>;

    // 6.3.2 + 6.4.2: онбординг в одной транзакции — клиент → счёт → карта +
    // первая транзакция, связи через RETURNING id. Возвращает номер счёта.
    fn onboard(
        &self,
        client: &mut Client,
        full_name: &str,
        email: &str,
        currency: &str,
        deposit: &str,
    ) -> Result<String, String>;
}

pub fn make(secure: bool) -> Box<dyn Commands> {
    if secure {
        Box::new(Secure)
    } else {
        Box::new(Insecure)
    }
}

// ---- общие помощники (без пользовательского ввода, инъекции не несут) ----

pub(crate) fn current_user(client: &mut Client) -> Result<String, String> {
    match client.query_one("SELECT current_user", &[]) {
        Ok(row) => Ok(row.get(0)),
        Err(error) => Err(error.to_string()),
    }
}

pub(crate) fn list_tables(client: &mut Client) -> Result<Vec<String>, String> {
    let query = "SELECT table_name FROM information_schema.tables \
                 WHERE table_schema = 'public' ORDER BY table_name";

    match client.query(query, &[]) {
        Ok(rows) => Ok(rows.iter().map(|row| row.get(0)).collect()),
        Err(error) => Err(error.to_string()),
    }
}

// Колонки таблицы из каталога, в порядке объявления. Имя таблицы передаётся
// параметром $1 — это запрос к information_schema, не к самой таблице.
pub(crate) fn columns(client: &mut Client, table: &str) -> Result<Vec<String>, String> {
    let query = "SELECT column_name FROM information_schema.columns \
                 WHERE table_schema = 'public' AND table_name = $1 \
                 ORDER BY ordinal_position";

    match client.query(query, &[&table]) {
        Ok(rows) => Ok(rows.iter().map(|row| row.get(0)).collect()),
        Err(error) => Err(error.to_string()),
    }
}

// Для защищённой версии: проверить, что колонка есть в каталоге, и собрать
// список всех колонок, приведённых к тексту (для единообразного чтения и
// сравнения с текстовым параметром). Возвращает (колонки, SELECT-список).
pub(crate) fn validated_columns(
    client: &mut Client,
    table: &str,
    column: &str,
) -> Result<(Vec<String>, String), String> {
    let columns = columns(client, table)?;
    if !columns.iter().any(|known| known == column) {
        return Err(format!("неизвестная колонка: {:?}", column));
    }

    let select_list = columns
        .iter()
        .map(|name| format!("\"{}\"::text", name))
        .collect::<Vec<_>>()
        .join(", ");

    Ok((columns, select_list))
}

// Колонки, которые пользователь должен заполнять при вставке: без DEFAULT и
// не IDENTITY. Автоматические (id, created_at, opened_at, expires_on и прочие
// с дефолтами) сюда не попадают — их проставляет БД.
pub(crate) fn insertable_columns(client: &mut Client, table: &str) -> Result<Vec<String>, String> {
    let query = "SELECT column_name FROM information_schema.columns \
                 WHERE table_schema = 'public' AND table_name = $1 \
                   AND is_identity = 'NO' AND column_default IS NULL \
                 ORDER BY ordinal_position";

    match client.query(query, &[&table]) {
        Ok(rows) => Ok(rows.iter().map(|row| row.get(0)).collect()),
        Err(error) => Err(error.to_string()),
    }
}

// Тип колонки для приведения значения (udt_name напрямую пригоден для ::cast:
// numeric, int4, int8, text, varchar, bpchar, timestamptz, date, bool).
pub(crate) fn column_type(
    client: &mut Client,
    table: &str,
    column: &str,
) -> Result<String, String> {
    let query = "SELECT udt_name FROM information_schema.columns \
                 WHERE table_schema = 'public' AND table_name = $1 AND column_name = $2";

    match client.query(query, &[&table, &column]) {
        Ok(rows) => match rows.first() {
            Some(row) => Ok(row.get(0)),
            None => Err(format!("неизвестная колонка: {:?}", column)),
        },
        Err(error) => Err(error.to_string()),
    }
}

// Число изменённых строк из ответа simple_query (сумма по CommandComplete).
pub(crate) fn affected(messages: Vec<SimpleQueryMessage>) -> u64 {
    messages
        .iter()
        .map(|message| match message {
            SimpleQueryMessage::CommandComplete(rows) => *rows,
            _ => 0,
        })
        .sum()
}

pub(crate) fn select_all(client: &mut Client, table: &str) -> Result<Table, String> {
    let query = format!("SELECT * FROM {}", table);

    match client.simple_query(&query) {
        Ok(messages) => Ok(text_table(messages)),
        Err(error) => Err(error.to_string()),
    }
}

// Строки из simple_query — все значения приходят текстом, типы колонок
// сопоставлять не нужно.
pub(crate) fn text_table(messages: Vec<SimpleQueryMessage>) -> Table {
    let mut headers: Vec<String> = Vec::new();
    let mut rows: Vec<Vec<String>> = Vec::new();

    for message in messages {
        if let SimpleQueryMessage::Row(row) = message {
            if headers.is_empty() {
                headers = row.columns().iter().map(|c| c.name().to_string()).collect();
            }

            let mut cells = Vec::with_capacity(row.len());
            for index in 0..row.len() {
                cells.push(row.get(index).unwrap_or("").to_string());
            }
            rows.push(cells);
        }
    }

    Table { headers, rows }
}

// Строки из обычного (расширенного) запроса, где все колонки уже приведены к
// тексту в SQL (::text). Используется защищённой версией с плейсхолдером.
pub(crate) fn typed_text_table(headers: &[String], rows: &[Row]) -> Table {
    let mut out: Vec<Vec<String>> = Vec::with_capacity(rows.len());

    for row in rows {
        let mut cells = Vec::with_capacity(headers.len());
        for index in 0..headers.len() {
            let cell: Option<String> = row.get(index);
            cells.push(cell.unwrap_or_default());
        }
        out.push(cells);
    }

    Table {
        headers: headers.to_vec(),
        rows: out,
    }
}
