// Защита от сценариев 1 и 2.
// Зависает на сценарии 3: путь исчез молча, ответ сервера не дойдёт.

use std::thread;
use std::time::Duration;

fn main() {
    let settings = app::settings();
    app::setup_logging(&settings.log_file);

    let mut pg = postgres::Config::new();
    pg.host(&settings.db_host);
    pg.port(settings.db_port);
    pg.dbname(&settings.db_name);
    pg.user(&settings.db_user);
    pg.password(&settings.db_password);
    pg.application_name("task2-fails3");
    pg.keepalives(false);

    pg.connect_timeout(Duration::from_secs(3));

    // Сценарий 2 закрыт. Лимит ставит сервер: он снимает собственный backend,
    // откатывает транзакцию и освобождает блокировки, затем присылает ошибку
    // 57014 query_canceled. Клиентский таймер так не умеет — он перестал бы
    // ждать, а запрос продолжил бы выполняться.
    pg.options("-c statement_timeout=5000");

    log::info!("fails3: connect_timeout + statement_timeout, интервал {} с", settings.ping_interval_seconds);

    let interval = Duration::from_secs(settings.ping_interval_seconds);

    loop {
        app::ping(&pg, &settings.query);
        thread::sleep(interval);
    }
}
