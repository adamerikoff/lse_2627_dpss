// Слой работы с БД. Db — живой дескриптор подключения (его держит App);
// connect() — разница secure/insecure на входе; операции над данными — в
// модуле commands. Журнал сюда не входит: логирование отдельно.

pub mod commands;
pub mod insecure;
pub mod secure;

use crate::db::commands::Commands;
use crate::db::commands::Table;
use postgres::Client;
use postgres::Config;
use postgres::NoTls;

// Живой дескриптор: адрес (из конфига), выбранные реализации подключения и
// команд, активное соединение и роль сессии. Создаётся при старте, соединение
// появляется после connect(). Логин и пароль приходят от пользователя (п. 6.6).
pub struct Db {
    secure: bool,
    host: String,
    port: u16,
    dbname: String,
    commands: Box<dyn Commands>,
    client: Option<Client>,
    pub role: String,
}

impl Db {
    pub fn new(secure: bool, host: String, port: u16, dbname: String) -> Db {
        Db {
            secure,
            host,
            port,
            dbname,
            commands: commands::make(secure),
            client: None,
            role: String::new(),
        }
    }

    pub fn secure(&self) -> bool {
        self.secure
    }

    // Подключение и запрос роли сессии (bank_employee или bank_admin).
    pub fn connect(&mut self, login: &str, password: &str) -> Result<(), String> {
        let mut client = connect(
            self.secure,
            &self.host,
            self.port,
            &self.dbname,
            login,
            password,
        )?;
        self.role = self.commands.current_user(&mut client)?;
        self.client = Some(client);
        Ok(())
    }

    pub fn tables(&mut self) -> Result<Vec<String>, String> {
        let client = match self.client.as_mut() {
            Some(client) => client,
            None => return Err(String::from("нет подключения")),
        };
        self.commands.list_tables(client)
    }

    pub fn select_all(&mut self, table: &str) -> Result<Table, String> {
        self.commands
            .select_all(self.client.as_mut().ok_or("нет подключения")?, table)
    }

    pub fn filter(&mut self, table: &str, column: &str, value: &str) -> Result<Table, String> {
        self.commands.filter(
            self.client.as_mut().ok_or("нет подключения")?,
            table,
            column,
            value,
        )
    }

    pub fn filter_in(
        &mut self,
        table: &str,
        column: &str,
        values: &[String],
    ) -> Result<Table, String> {
        self.commands.filter_in(
            self.client.as_mut().ok_or("нет подключения")?,
            table,
            column,
            values,
        )
    }

    pub fn update(
        &mut self,
        table: &str,
        set_column: &str,
        set_value: &str,
        where_column: &str,
        where_value: &str,
    ) -> Result<u64, String> {
        self.commands.update(
            self.client.as_mut().ok_or("нет подключения")?,
            table,
            set_column,
            set_value,
            where_column,
            where_value,
        )
    }

    pub fn update_in(
        &mut self,
        table: &str,
        set_column: &str,
        set_value: &str,
        where_column: &str,
        where_values: &[String],
    ) -> Result<u64, String> {
        self.commands.update_in(
            self.client.as_mut().ok_or("нет подключения")?,
            table,
            set_column,
            set_value,
            where_column,
            where_values,
        )
    }

    pub fn insert(
        &mut self,
        table: &str,
        columns: &[String],
        values: &[String],
    ) -> Result<u64, String> {
        self.commands.insert(
            self.client.as_mut().ok_or("нет подключения")?,
            table,
            columns,
            values,
        )
    }

    pub fn onboard(
        &mut self,
        full_name: &str,
        email: &str,
        currency: &str,
        deposit: &str,
    ) -> Result<String, String> {
        self.commands.onboard(
            self.client.as_mut().ok_or("нет подключения")?,
            full_name,
            email,
            currency,
            deposit,
        )
    }

    // Колонки для формы вставки: только те, что должен заполнить пользователь
    // (без DEFAULT и не IDENTITY). Остальное проставит БД.
    pub fn insertable_columns(&mut self, table: &str) -> Result<Vec<String>, String> {
        commands::insertable_columns(self.client.as_mut().ok_or("нет подключения")?, table)
    }
}

fn connect(
    secure: bool,
    host: &str,
    port: u16,
    dbname: &str,
    login: &str,
    password: &str,
) -> Result<Client, String> {
    if secure {
        connect_secure(host, port, dbname, login, password)
    } else {
        connect_insecure(host, port, dbname, login, password)
    }
}

// УЯЗВИМАЯ: строка подключения собирается конкатенацией. Логин и пароль
// попадают прямо в текст conninfo, поэтому ввод вида
//   bank_employee options='-c role=bank_admin'
// добавляет параметр options и переключает роль сессии — connection-string
// инъекция, вход как bank_admin без его пароля.
fn connect_insecure(
    host: &str,
    port: u16,
    dbname: &str,
    login: &str,
    password: &str,
) -> Result<Client, String> {
    let conninfo = format!(
        "host={} port={} dbname={} user={} password={}",
        host, port, dbname, login, password
    );

    match Client::connect(&conninfo, NoTls) {
        Ok(client) => Ok(client),
        Err(error) => Err(error.to_string()),
    }
}

// ЗАЩИЩЁННАЯ: значения кладутся в отдельные поля Config и уходят в протокол
// как литералы. Инъекция становится частью логина/пароля — аутентификация
// проваливается, параметром подключения ввод не станет.
fn connect_secure(
    host: &str,
    port: u16,
    dbname: &str,
    login: &str,
    password: &str,
) -> Result<Client, String> {
    let mut config = Config::new();
    config
        .host(host)
        .port(port)
        .dbname(dbname)
        .user(login)
        .password(password);

    match config.connect(NoTls) {
        Ok(client) => Ok(client),
        Err(error) => Err(error.to_string()),
    }
}
