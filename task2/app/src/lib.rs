// Общая скучная часть: конфигурация и логирование. Сами сценарии — в
// src/bin/fails*.rs, по файлу на экземпляр.

use config::{Config, Environment, File};
use serde::Deserialize;
use std::env;

// config.toml (упакован в образ, п. 1.8) плюс переменные APP_*, которые его
// переопределяют: db_host -> APP_DB_HOST.
#[derive(Deserialize)]
pub struct Settings {
    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,
    pub ping_interval_seconds: u64,
    pub query: String,
    pub log_file: Option<String>,
}

pub fn settings() -> Settings {
    let exe = env::current_exe().expect("не найти свой исполняемый файл");

    let sources = Config::builder()
        .add_source(File::from(exe.with_file_name("config.toml")))
        .add_source(Environment::with_prefix("APP").ignore_empty(true).try_parsing(true))
        .build()
        .expect("не собрать конфигурацию");

    sources.try_deserialize().expect("не разобрать конфигурацию")
}

// log::info! -> stdout, log::error! -> stderr (п. 1.2). Если задан
// APP_LOG_FILE — обе категории дублируются в файл (п. 1.7).
pub fn setup_logging(log_file: &Option<String>) {
    let mut to_stdout = fern::Dispatch::new();
    // В крейте log порядок по строгости: Error < Warn < Info.
    to_stdout = to_stdout.filter(|metadata| metadata.level() >= log::Level::Info);
    to_stdout = to_stdout.chain(std::io::stdout());

    let mut to_stderr = fern::Dispatch::new();
    to_stderr = to_stderr.level(log::LevelFilter::Warn);
    to_stderr = to_stderr.chain(std::io::stderr());

    let mut root = fern::Dispatch::new();
    // Драйвер postgres пишет в тот же фасад log на уровне TRACE. Без этой
    // строки его внутренности забивают весь вывод.
    root = root.level(log::LevelFilter::Info);
    root = root.format(|out, message, record| {
        let stamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        out.finish(format_args!("{} {:5} {}", stamp, record.level(), message));
    });
    root = root.chain(to_stdout);
    root = root.chain(to_stderr);

    if let Some(path) = log_file {
        // fern::log_file только открывает файл, каталог создаём сами.
        let directory = std::path::Path::new(path).parent().expect("APP_LOG_FILE без каталога");
        std::fs::create_dir_all(directory).expect("не создать каталог для APP_LOG_FILE");
        root = root.chain(fern::log_file(path).expect("не открыть APP_LOG_FILE"));
    }

    // Провалиться может только если логгер уже установлен, то есть никогда.
    let _ = root.apply();
}

// Драйвер прячет причину в источнике ошибки: сам postgres::Error говорит лишь
// "error connecting to server", а под ним лежит "Connection refused" или
// "Operation timed out". Без этого сценарии 0 и 1 в логе неразличимы.
fn describe(error: &dyn std::error::Error) -> String {
    let mut text = error.to_string();
    let mut source = error.source();

    while let Some(inner) = source {
        text.push_str(": ");
        text.push_str(&inner.to_string());
        source = inner.source();
    }

    text
}

// Одна проверка: подключиться, выполнить запрос, записать исход.
// Успех и нетипичный ответ — в stdout, отказ — в stderr.
pub fn ping(pg: &postgres::Config, query: &str) {
    let began = std::time::Instant::now();

    let mut client = match pg.connect(postgres::NoTls) {
        Ok(client) => client,
        Err(error) => {
            log::error!("подключение: {} ({} мс)", describe(&error), began.elapsed().as_millis());
            return;
        }
    };

    let rows = match client.query(query, &[]) {
        Ok(rows) => rows,
        Err(error) => {
            log::error!("запрос: {} ({} мс)", describe(&error), began.elapsed().as_millis());
            return;
        }
    };

    // Ответ без ошибки, но не такой, как ожидалось (п. 1.3): stdout, не stderr.
    if rows.len() != 1 {
        log::info!("нетипичный ответ: строк {}, ожидалась одна ({} мс)", rows.len(), began.elapsed().as_millis());
        return;
    }

    let column: Result<String, postgres::Error> = rows[0].try_get(0);

    let version = match column {
        Ok(version) => version,
        Err(error) => {
            log::info!("нетипичный ответ: столбец не текстовый: {} ({} мс)", error, began.elapsed().as_millis());
            return;
        }
    };

    if !version.starts_with("PostgreSQL") {
        log::info!("нетипичный ответ: {:?} ({} мс)", version, began.elapsed().as_millis());
        return;
    }

    log::info!("успех: {} ({} мс)", version, began.elapsed().as_millis());
}
