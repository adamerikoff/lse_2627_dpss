// Защита от сценария 1: свой дедлайн на подключение вместо 127 секунд ядра.
// Зависает на сценарии 2: сервер принял запрос и не отвечает.

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
    pg.application_name("task2-fails2");
    pg.keepalives(false);

    // Сценарий 1 закрыт. Дедлайн охватывает TCP-рукопожатие и аутентификацию,
    // но не выполнение запроса.
    pg.connect_timeout(Duration::from_secs(3));

    log::info!("fails2: connect_timeout, интервал {} с", settings.ping_interval_seconds);

    let interval = Duration::from_secs(settings.ping_interval_seconds);

    loop {
        app::ping(&pg, &settings.query);
        thread::sleep(interval);
    }
}
