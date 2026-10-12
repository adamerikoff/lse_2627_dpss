// Защита от сценариев 1, 2 и 3.
// Зависает на сценарии 4: сервер заморожен, но его ядро отвечает на пробы.

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
    pg.application_name("task2-fails4");

    pg.connect_timeout(Duration::from_secs(3));
    pg.options("-c statement_timeout=5000");

    // Сценарий 3 закрыт. Одного idle недостаточно: по умолчанию пробы идут с
    // интервалом 75 секунд и повторяются 9 раз, то есть ещё 11 минут. Поэтому
    // заданы все четыре параметра.
    let idle = Duration::from_secs(10);
    let probe_interval = Duration::from_secs(5);
    let retries = 3;

    pg.keepalives(true);
    pg.keepalives_idle(idle);
    pg.keepalives_interval(probe_interval);
    pg.keepalives_retries(retries);

    // RFC 5482: потолок на неподтверждённые данные, не зависящий от
    // interval * retries. В Linux ограничивает и фазу keepalive-проб.
    pg.tcp_user_timeout(idle + probe_interval * retries);

    log::info!("fails4: + keepalive, интервал {} с", settings.ping_interval_seconds);

    let interval = Duration::from_secs(settings.ping_interval_seconds);

    loop {
        app::ping(&pg, &settings.query);
        thread::sleep(interval);
    }
}
