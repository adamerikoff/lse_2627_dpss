// Защищённые операции: значения — плейсхолдером $1, имена колонок — из белого
// списка каталога. Инъекция в значение гаснет (параметр), инъекция в имя
// колонки отсекается (нет в каталоге).

use crate::db::commands;
use crate::db::commands::Commands;
use crate::db::commands::Table;
use postgres::Client;
use postgres::types::ToSql;

pub struct Secure;

impl Commands for Secure {
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
        // Имя колонки — только из каталога; значение — параметром $1.
        let (columns, select_list) = commands::validated_columns(client, table, column)?;

        let sql = format!(
            "SELECT {} FROM \"{}\" WHERE \"{}\"::text = $1",
            select_list, table, column
        );

        match client.query(&sql, &[&value]) {
            Ok(rows) => Ok(commands::typed_text_table(&columns, &rows)),
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
        // Группа значений — одним параметром-массивом text[]; число значений
        // на текст запроса не влияет, вклеивать нечего.
        let (columns, select_list) = commands::validated_columns(client, table, column)?;

        let sql = format!(
            "SELECT {} FROM \"{}\" WHERE \"{}\"::text = ANY($1)",
            select_list, table, column
        );

        let owned: Vec<String> = values.to_vec();

        match client.query(&sql, &[&owned]) {
            Ok(rows) => Ok(commands::typed_text_table(&columns, &rows)),
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
        let set_type = check_set(client, table, set_column, where_column)?;

        // SET через $1, приведённый к типу колонки; условие — $2 как текст.
        let sql = format!(
            "UPDATE \"{}\" SET \"{}\" = $1::{} WHERE \"{}\"::text = $2",
            table, set_column, set_type, where_column
        );

        client
            .execute(&sql, &[&set_value, &where_value])
            .map_err(|error| error.to_string())
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
        let set_type = check_set(client, table, set_column, where_column)?;

        let sql = format!(
            "UPDATE \"{}\" SET \"{}\" = $1::{} WHERE \"{}\"::text = ANY($2)",
            table, set_column, set_type, where_column
        );

        let owned: Vec<String> = where_values.to_vec();
        client
            .execute(&sql, &[&set_value, &owned])
            .map_err(|error| error.to_string())
    }

    fn insert(
        &self,
        client: &mut Client,
        table: &str,
        columns: &[String],
        values: &[String],
    ) -> Result<u64, String> {
        if columns.is_empty() {
            return Err(String::from("нет значений для вставки"));
        }

        // Тип каждой колонки — для приведения текстового значения.
        let mut types = Vec::with_capacity(columns.len());
        for column in columns {
            types.push(commands::column_type(client, table, column)?);
        }

        let names = columns
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<_>>()
            .join(", ");

        let placeholders = (0..columns.len())
            .map(|i| format!("${}::{}", i + 1, types[i]))
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            "INSERT INTO \"{}\" ({}) VALUES ({})",
            table, names, placeholders
        );

        let params: Vec<&(dyn ToSql + Sync)> = values
            .iter()
            .map(|value| value as &(dyn ToSql + Sync))
            .collect();

        client
            .execute(&sql, &params)
            .map_err(|error| error.to_string())
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

        // Всё в одной транзакции: любой сбой откатывает целиком.
        let mut tx = client.transaction().map_err(|error| error.to_string())?;

        let customer = tx
            .query_one(
                "INSERT INTO customers (full_name, email) VALUES ($1, $2) RETURNING id",
                &[&full_name, &email],
            )
            .map_err(|error| error.to_string())?;
        let customer_id: i64 = customer.get(0);

        let generated = tx
            .query_one(
                "SELECT 'ACC-' || lpad((count(*) + 1)::text, 4, '0') FROM accounts",
                &[],
            )
            .map_err(|error| error.to_string())?;
        let account_number: String = generated.get(0);

        let account = tx
            .query_one(
                "INSERT INTO accounts (customer_id, account_number, currency, balance) \
                 VALUES ($1, $2, $3, $4::numeric) RETURNING id",
                &[&customer_id, &account_number, &currency, &deposit],
            )
            .map_err(|error| error.to_string())?;
        let account_id: i64 = account.get(0);

        let suffix = &account_number[account_number.len().saturating_sub(4)..];
        let card_number = format!("4111-0000-0000-{}", suffix);

        // expires_on проставит DEFAULT (+4 года).
        tx.execute(
            "INSERT INTO cards (account_id, card_number, holder_name) VALUES ($1, $2, $3)",
            &[&account_id, &card_number, &holder],
        )
        .map_err(|error| error.to_string())?;

        tx.execute(
            "INSERT INTO transactions (account_id, kind, amount, note) \
             VALUES ($1, 'deposit', $2::numeric, 'первое пополнение')",
            &[&account_id, &deposit],
        )
        .map_err(|error| error.to_string())?;

        tx.commit().map_err(|error| error.to_string())?;
        Ok(account_number)
    }
}

// Общая проверка для UPDATE: обе колонки из каталога, менять id нельзя
// (п. 6.2.1). Возвращает тип set-колонки для приведения значения.
fn check_set(
    client: &mut Client,
    table: &str,
    set_column: &str,
    where_column: &str,
) -> Result<String, String> {
    if set_column == "id" {
        return Err(String::from("нельзя менять id"));
    }

    let columns = commands::columns(client, table)?;
    if !columns.iter().any(|known| known == set_column) {
        return Err(format!("неизвестная колонка: {:?}", set_column));
    }
    if !columns.iter().any(|known| known == where_column) {
        return Err(format!("неизвестная колонка: {:?}", where_column));
    }

    commands::column_type(client, table, set_column)
}
