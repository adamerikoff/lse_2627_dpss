// Конфигурация из окружения с дефолтами. .env (если есть) подхватывается
// автоматически — это и есть Env-файл из п. 6.10. Переменные с префиксом APP_:
//   APP_DB_HOST  APP_DB_PORT  APP_DB_NAME  APP_LOG_FILE

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_db_host")]
    pub db_host: String,
    #[serde(default = "default_db_port")]
    pub db_port: u16,
    #[serde(default = "default_db_name")]
    pub db_name: String,
    #[serde(default = "default_log_file")]
    pub log_file: String,
}

pub fn load() -> Config {
    // Загружаем .env в окружение, если файл есть; его отсутствие — не ошибка.
    let _ = dotenvy::dotenv();

    // Непарсящееся значение (например, нечисловой порт) — ошибка конфигурации,
    // падаем сразу с понятным текстом, а не работаем с мусором.
    match envy::prefixed("APP_").from_env::<Config>() {
        Ok(config) => config,
        Err(error) => panic!("ошибка конфигурации: {}", error),
    }
}

fn default_db_host() -> String {
    String::from("127.0.0.1")
}

fn default_db_port() -> u16 {
    55434
}

fn default_db_name() -> String {
    String::from("neobank")
}

fn default_log_file() -> String {
    String::from("logs/system.log")
}
