// Уязвимые операции: имя колонки и значение вклеиваются в текст запроса
// конкатенацией. simple_query вдобавок допускает несколько операторов через
// `;`, поэтому инъекция здесь максимально широкая.

use crate::db::commands;
use crate::db::commands::Commands;
use crate::db::commands::Table;
use postgres::Client;
use postgres::SimpleQueryMessage;
use postgres::Transaction;

pub struct Insecure;

impl Commands for Insecure {
    fn current_user(&self, client: &mut Client) -> Result<String, String> {
        commands::current_user(client)
    }

    fn list_tables(&self, client: &mut Client) -> Result<Vec<String>, String> {
        commands::list_tables(client)
    }

    fn select_all(&self, client: &mut Client, table: &str) -> Result<Table, String> {
        commands::select_all(client, table)
    }

    fn filter(
        &self,
        client: &mut Client,
        table: &str,
        column: &str,
        value: &str,
    ) -> Result<Table, String> {
        // И имя колонки, и значение — прямо в текст. Нагрузки:
        //   значение: ' OR '1'='1
        //   колонка:  name='x' OR 1=1 --
        let sql = format!("SELECT * FROM {} WHERE {} = '{}'", table, column, value);

        match client.simple_query(&sql) {
            Ok(messages) => Ok(commands::text_table(messages)),
            Err(error) => Err(error.to_string()),
        }
    }

    fn filter_in(
        &self,
        client: &mut Client,
        table: &str,
        column: &str,
        values: &[String],
    ) -> Result<Table, String> {
        // Значения склеиваются в список через запятую, каждое в кавычках.
        // Инъекция и в имя колонки, и в любое из значений.
        let sql = format!(
            "SELECT * FROM {} WHERE {} IN ({})",
            table,
            column,
            quoted(values)
        );

        match client.simple_query(&sql) {
            Ok(messages) => Ok(commands::text_table(messages)),
            Err(error) => Err(error.to_string()),
        }
    }

    fn update(
        &self,
        client: &mut Client,
        table: &str,
        set_column: &str,
        set_value: &str,
        where_column: &str,
        where_value: &str,
    ) -> Result<u64, String> {
        // Всё в текст. Инъекция в where-значение (' OR '1'='1) обновит все
        // строки; simple_query вдобавок пропускает `;` с другим оператором.
        let sql = format!(
            "UPDATE {} SET {} = '{}' WHERE {} = '{}'",
            table, set_column, set_value, where_column, where_value
        );

        match client.simple_query(&sql) {
            Ok(messages) => Ok(commands::affected(messages)),
            Err(error) => Err(error.to_string()),
        }
    }

    fn update_in(
        &self,
        client: &mut Client,
        table: &str,
        set_column: &str,
        set_value: &str,
        where_column: &str,
        where_values: &[String],
    ) -> Result<u64, String> {
        let sql = format!(
            "UPDATE {} SET {} = '{}' WHERE {} IN ({})",
            table,
            set_column,
            set_value,
            where_column,
            quoted(where_values)
        );

        match client.simple_query(&sql) {
            Ok(messages) => Ok(commands::affected(messages)),
            Err(error) => Err(error.to_string()),
        }
    }

    fn insert(
        &self,
        client: &mut Client,
        table: &str,
        columns: &[String],
        values: &[String],
    ) -> Result<u64, String> {
        // Значения вклеиваются в кавычках — инъекция в любом из них.
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            columns.join(", "),
            quoted(values)
        );

        match client.simple_query(&sql) {
            Ok(messages) => Ok(commands::affected(messages)),
            Err(error) => Err(error.to_string()),
        }
    }

    fn onboard(
        &self,
        client: &mut Client,
        full_name: &str,
        email: &str,
        currency: &str,
        deposit: &str,
    ) -> Result<String, String> {
        let holder = full_name.to_uppercase();

        let mut tx = client.transaction().map_err(|error| error.to_string())?;

        // Каждый оператор собран конкатенацией — инъекция в любом текстовом
        // поле; simple_query к тому же пропускает несколько операторов.
        let customer_id = scalar(
            &mut tx,
            &format!(
                "INSERT INTO customers (full_name, email) VALUES ('{}', '{}') RETURNING id",
                full_name, email
            ),
        )?;

        let account_number = scalar(
            &mut tx,
            "SELECT 'ACC-' || lpad((count(*) + 1)::text, 4, '0') FROM accounts",
        )?;

        let account_id = scalar(
            &mut tx,
            &format!(
                "INSERT INTO accounts (customer_id, account_number, currency, balance) \
                 VALUES ({}, '{}', '{}', {}) RETURNING id",
                customer_id, account_number, currency, deposit
            ),
        )?;

        let suffix = &account_number[account_number.len().saturating_sub(4)..];
        let card_number = format!("4111-0000-0000-{}", suffix);

        // expires_on проставит DEFAULT (+4 года).
        tx.simple_query(&format!(
            "INSERT INTO cards (account_id, card_number, holder_name) VALUES ({}, '{}', '{}')",
            account_id, card_number, holder
        ))
        .map_err(|error| error.to_string())?;

        tx.simple_query(&format!(
            "INSERT INTO transactions (account_id, kind, amount, note) \
             VALUES ({}, 'deposit', {}, 'первое пополнение')",
            account_id, deposit
        ))
        .map_err(|error| error.to_string())?;

        tx.commit().map_err(|error| error.to_string())?;
        Ok(account_number)
    }
}

// Значения в кавычках через запятую для списка VALUES/IN. Экранирования нет —
// в этом и состоит уязвимость.
fn quoted(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("'{}'", value))
        .collect::<Vec<_>>()
        .join(", ")
}

// Первое скалярное значение (как текст) из ответа simple_query.
fn scalar(tx: &mut Transaction, sql: &str) -> Result<String, String> {
    let messages = tx.simple_query(sql).map_err(|error| error.to_string())?;

    for message in messages {
        if let SimpleQueryMessage::Row(row) = message
            && let Some(value) = row.get(0)
        {
            return Ok(value.to_string());
        }
    }

    Err(String::from("пустой результат"))
}
