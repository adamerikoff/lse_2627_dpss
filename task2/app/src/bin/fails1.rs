// Ни одной защиты. Зависает на сценарии 1: ядро повторяет SYN около 127 секунд.

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
    pg.application_name("task2-fails1");

    // keepalives в драйвере включён по умолчанию, но системный idle равен
    // 7200 секундам — защиты фактически нет. Выключаем явно, чтобы это было
    // видно в коде.
    pg.keepalives(false);

    log::info!("fails1: защит нет, интервал {} с", settings.ping_interval_seconds);

    let interval = Duration::from_secs(settings.ping_interval_seconds);

    loop {
        app::ping(&pg, &settings.query);
        thread::sleep(interval);
    }
}
